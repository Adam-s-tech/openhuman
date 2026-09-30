use super::*;
use crate::inference::provider::UsageInfo;

#[tokio::test]
async fn scoped_tool_limits_narrow_restore_and_keep_zero() {
    assert_eq!(tool_call_limit(10), 80);
    with_tool_call_limit(Some(4), async {
        assert_eq!(tool_call_limit(10), 4);
        with_tool_call_limit(Some(12), async {
            assert_eq!(tool_call_limit(10), 4);
        })
        .await;
        with_tool_call_limit(Some(0), async {
            assert_eq!(tool_call_limit(10), 0);
        })
        .await;
        assert_eq!(tool_call_limit(10), 4);
        with_tool_call_limit(None, async {
            assert_eq!(tool_call_limit(10), 4);
        })
        .await;
    })
    .await;
    assert_eq!(tool_call_limit(10), 80);
    with_tool_call_limit(Some(100), async {
        assert_eq!(tool_call_limit(1), 8);
    })
    .await;
}

#[tokio::test]
async fn concurrent_turns_do_not_share_tool_limits() {
    let (a, b) = tokio::join!(
        with_tool_call_limit(Some(2), async {
            tokio::task::yield_now().await;
            tool_call_limit(10)
        }),
        with_tool_call_limit(Some(7), async {
            tokio::task::yield_now().await;
            tool_call_limit(10)
        })
    );
    assert_eq!((a, b), (2, 7));
    assert_eq!(tool_call_limit(10), 80);
}

#[tokio::test]
async fn cancelled_scope_restores_the_callers_tool_limit() {
    let mut scoped = Box::pin(with_tool_call_limit(Some(0), async {
        assert_eq!(tool_call_limit(10), 0);
        std::future::pending::<()>().await;
    }));
    assert!(futures::poll!(&mut scoped).is_pending());
    drop(scoped);
    assert_eq!(tool_call_limit(10), 80);
}
