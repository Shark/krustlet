use json_patch::{PatchOperation, ReplaceOperation};
use crate::{PodState, ProviderState};
use kubelet::pod::state::prelude::*;
use kubelet::state::common::GenericProviderState;
use anyhow::anyhow;

use k8s_openapi::api::core::v1::ConfigMap;
use kube::Api;
use kube::api::{Patch, PatchParams};

/// Pod was deleted.
#[derive(Default, Debug)]
pub struct Completed;

#[async_trait::async_trait]
impl State<PodState> for Completed {
    async fn next(
        self: Box<Self>,
        provider_state: SharedState<ProviderState>,
        pod_state: &mut PodState,
        pod: Manifest<Pod>,
    ) -> Transition<PodState> {
        let result = match pod_state.pod_working_dir.result() {
            Ok(r) => r,
            Err(why) => return Transition::Complete(Err(anyhow!(why).context("Error reading module result from working dir"))),
        };
        let result = match serde_json::to_string(&result) {
            Ok(r) => r,
            Err(why) => return Transition::Complete(Err(why.into())),
        };
        tracing::debug!(?result, "Retrieved result");
        let client = {
            let provider_state = provider_state.read().await;
            provider_state.client()
        };
        let (pod_name, pod_namespace) = {
            let pod = pod.latest();
            (pod.name().to_owned(), pod.namespace().to_owned())
        };
        let api: Api<ConfigMap> = Api::namespaced(client.clone(), &pod_namespace);
        let patch = {
            let json_patch = json_patch::Patch(vec![
                PatchOperation::Replace(ReplaceOperation {
                    path: "/data/result.json".to_string(),
                    value: serde_json::Value::String(result),
                }),
            ]);
            Patch::Json::<()>(json_patch)
        };
        match api.patch(&pod_name, &PatchParams::default(), &patch).await {
            Ok(_) => tracing::debug!(?pod_name, "Patched Pod"),
            Err(why) => {
                tracing::error!(?pod_name, ?why, "Error applying Pod patch");
                return Transition::Complete(Err(why.into()))
            }
        }
        Transition::Complete(Ok(()))
    }

    async fn status(&self, _pod_state: &mut PodState, _pod: &Pod) -> anyhow::Result<PodStatus> {
        Ok(make_status(Phase::Succeeded, "Completed"))
    }
}
