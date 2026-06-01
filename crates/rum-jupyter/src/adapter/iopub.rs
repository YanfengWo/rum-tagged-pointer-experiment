use super::*;

impl KernelAdapter {
    pub(super) fn execute_prologue_iopub(
        &mut self,
        parent_header: &KernelMessageHeader,
        execution_count: Option<u64>,
        code: &str,
    ) -> Vec<KernelIoPubEnvelope> {
        let mut iopub = vec![self.status_iopub(parent_header, KernelExecutionState::Busy)];

        if execution_count.is_some() {
            iopub.push(self.execute_input_iopub(parent_header, execution_count, code));
        }

        iopub
    }

    pub(super) fn status_iopub(
        &mut self,
        parent_header: &KernelMessageHeader,
        execution_state: KernelExecutionState,
    ) -> KernelIoPubEnvelope {
        let mut metadata = KernelMetadata::new();
        metadata.insert("msg_type", "status");
        metadata.insert("channel", KernelChannel::IoPub.as_str());
        metadata.insert("execution_state", execution_state.as_str());
        metadata.insert("lifecycle_state", self.lifecycle_state.as_str());
        metadata.insert("parent_msg_type", &parent_header.msg_type);

        KernelIoPubEnvelope {
            channel: KernelChannel::IoPub,
            header: self.next_response_header(parent_header, "status"),
            parent_header: Some(parent_header.clone()),
            metadata,
            payload: KernelIoPubPayload::Status(KernelStatusOutput {
                execution_state,
                lifecycle_state: self.lifecycle_state,
            }),
        }
    }

    pub(super) fn execute_input_iopub(
        &mut self,
        parent_header: &KernelMessageHeader,
        execution_count: Option<u64>,
        code: &str,
    ) -> KernelIoPubEnvelope {
        let mut metadata = KernelMetadata::new();
        metadata.insert("msg_type", "execute_input");
        metadata.insert("channel", KernelChannel::IoPub.as_str());
        metadata.insert("lifecycle_state", self.lifecycle_state.as_str());
        metadata.insert("parent_msg_type", &parent_header.msg_type);
        metadata.insert(
            "execution_count",
            execution_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "none".to_string()),
        );

        KernelIoPubEnvelope {
            channel: KernelChannel::IoPub,
            header: self.next_response_header(parent_header, "execute_input"),
            parent_header: Some(parent_header.clone()),
            metadata,
            payload: KernelIoPubPayload::ExecuteInput(KernelExecuteInput {
                execution_count,
                code: code.to_string(),
            }),
        }
    }

    pub(super) fn output_iopub(
        &mut self,
        parent_header: &KernelMessageHeader,
        output: &KernelOutput,
    ) -> KernelIoPubEnvelope {
        let (msg_type, mut metadata, payload) = match output {
            KernelOutput::Stream(stream) => {
                let mut metadata = KernelMetadata::new();
                metadata.insert("msg_type", "stream");
                metadata.insert(
                    "name",
                    match stream.name {
                        KernelStreamName::Stdout => "stdout",
                        KernelStreamName::Stderr => "stderr",
                    },
                );
                (
                    "stream",
                    metadata,
                    KernelIoPubPayload::Stream(stream.clone()),
                )
            }
            KernelOutput::ExecuteResult(result) => (
                "execute_result",
                result.metadata.clone(),
                KernelIoPubPayload::ExecuteResult(result.clone()),
            ),
            KernelOutput::DisplayData(display) => (
                "display_data",
                display.metadata.clone(),
                KernelIoPubPayload::DisplayData(display.clone()),
            ),
            KernelOutput::Error(error) => {
                let mut metadata = KernelMetadata::new();
                metadata.insert("msg_type", "error");
                metadata.insert("error_name", &error.error_name);
                ("error", metadata, KernelIoPubPayload::Error(error.clone()))
            }
        };
        metadata.insert("channel", KernelChannel::IoPub.as_str());
        metadata.insert("lifecycle_state", self.lifecycle_state.as_str());
        metadata.insert("parent_msg_type", &parent_header.msg_type);

        KernelIoPubEnvelope {
            channel: KernelChannel::IoPub,
            header: self.next_response_header(parent_header, msg_type),
            parent_header: Some(parent_header.clone()),
            metadata,
            payload,
        }
    }
}
