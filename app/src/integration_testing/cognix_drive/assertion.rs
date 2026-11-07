use crate::integration_testing::view_getters::workspace_view;
use cognixui::async_assert;
use cognixui::integration::AssertionCallback;

pub fn assert_workflow_modal_is_open() -> AssertionCallback {
    Box::new(move |app, window_id| {
        let workspace = workspace_view(app, window_id);

        workspace.read(app, |workspace, _| {
            async_assert!(
                workspace.is_workflow_modal_open(),
                "Expected workflow modal to be open, but it was closed"
            )
        })
    })
}

pub fn assert_workflow_modal_is_closed() -> AssertionCallback {
    Box::new(move |app, window_id| {
        let workspace = workspace_view(app, window_id);

        workspace.read(app, |workspace, _| {
            async_assert!(
                !workspace.is_workflow_modal_open(),
                "Expected workflow modal to be closed, but it was open"
            )
        })
    })
}

pub fn assert_cognix_drive_is_open() -> AssertionCallback {
    Box::new(move |app, window_id| {
        let workspace = workspace_view(app, window_id);

        workspace.read(app, |workspace, _| {
            async_assert!(
                workspace.is_cognix_drive_open(),
                "Expected Cognix Drive to be open, but it was closed"
            )
        })
    })
}

pub fn assert_cognix_drive_is_closed() -> AssertionCallback {
    Box::new(move |app, window_id| {
        let workspace = workspace_view(app, window_id);

        workspace.read(app, |workspace, _| {
            async_assert!(
                !workspace.is_cognix_drive_open(),
                "Expected Cognix Drive to be closed, but it was open"
            )
        })
    })
}

pub fn assert_is_left_panel_open() -> AssertionCallback {
    Box::new(move |app, window_id| {
        let workspace = workspace_view(app, window_id);

        workspace.read(app, |workspace, ctx| {
            async_assert!(
                workspace.is_left_panel_open(ctx),
                "Expected left panel to be open, but it was closed"
            )
        })
    })
}
