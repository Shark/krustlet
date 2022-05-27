use std::path::PathBuf;
use crate::ModuleRunContext;
use crate::ProviderState;
use krator::{ObjectState, SharedState};
use tracing::Span;
use kubelet::container::{Container, ContainerKey, Status};
use kubelet::pod::Pod;

pub(crate) mod running;
pub(crate) mod terminated;
pub(crate) mod waiting;

pub(crate) struct ContainerState {
    pod: Pod,
    container_key: ContainerKey,
    run_context: SharedState<ModuleRunContext>,
    pod_working_dir: PathBuf,
    parent_span: Span,
}

impl ContainerState {
    pub fn new(
        pod: Pod,
        container_key: ContainerKey,
        run_context: SharedState<ModuleRunContext>,
        pod_working_dir: PathBuf,
        parent_span: Span,
    ) -> Self {
        ContainerState {
            pod,
            container_key,
            run_context,
            pod_working_dir,
            parent_span,
        }
    }
}

#[async_trait::async_trait]
impl ObjectState for ContainerState {
    type Manifest = Container;
    type Status = Status;
    type SharedState = ProviderState;
    async fn async_drop(self, _shared_state: &mut Self::SharedState) {}
}
