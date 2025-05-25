pub enum RequestMethod {
    BuildInitialized,
    BuildTargetSources,
    BuildTargetDidChange,
    BuildTargetPrepare,
    BuildShutDown,
    BuildExit,
    WorkspaceBuildTargets,
    WorkspaceWaitForBuildSystemUpdates,
    WindowShowMessage,
    TextDocumentSourceKitOptions,
    TextDocumentRegisterForChanges,
    Unknown,
}

impl RequestMethod {
    pub fn from_str(s: &str) -> Self {
        match s {
            "build/initialized" => Self::BuildInitialized,
            "buildTarget/sources" => Self::BuildTargetSources,
            "buildTarget/didChange" => Self::BuildTargetDidChange,
            "buildTarget/prepare" => Self::BuildTargetPrepare,
            "build/shutdown" => Self::BuildShutDown,
            "build/exit" => Self::BuildExit,
            "workspace/buildTargets" => Self::WorkspaceBuildTargets,
            "workspace/waitForBuildSystemUpdates" => Self::WorkspaceWaitForBuildSystemUpdates,
            "window/showMessage" => Self::WindowShowMessage,
            "textDocument/sourceKitOptions" => Self::TextDocumentSourceKitOptions,
            "textDocument/registerForChanges" => Self::TextDocumentRegisterForChanges,
            _ => Self::Unknown,
        }
    }
}
