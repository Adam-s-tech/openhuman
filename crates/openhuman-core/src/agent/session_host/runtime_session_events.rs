//! Drain integration and skill change notifications at the turn boundary.
use super::*;

impl OpenHumanTurnPrelude {
    pub(super) fn drain_host_events(&self) -> bool {
        let mut mutable = self
            .mutable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if mutable.composio_events.is_none() {
            mutable.composio_events = crate::core::bus::BUS.get().map(|bus| bus.receiver());
        }
        if mutable.skill_events.is_none() {
            mutable.skill_events = crate::core::bus::BUS.get().map(|bus| bus.receiver());
        }
        let drain = |receiver: &mut Option<
            tinybus::events::EventReceiver<crate::core::events::DomainEvent>,
        >| {
            let Some(receiver) = receiver.as_mut() else {
                return false;
            };
            let mut skills_changed = false;
            loop {
                use tinybus::TryRecvError;
                match receiver.try_recv() {
                    Ok(crate::core::events::DomainEvent::WorkflowsChanged { .. }) => {
                        skills_changed = true
                    }
                    Ok(crate::core::events::DomainEvent::ComposioIntegrationsChanged {
                        ..
                    })
                    | Ok(_) => {}
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Lagged(_)) => {
                        skills_changed = true;
                        break;
                    }
                    Err(TryRecvError::Closed) => break,
                }
            }
            skills_changed
        };
        let composio_skills_changed = drain(&mut mutable.composio_events);
        composio_skills_changed | drain(&mut mutable.skill_events)
    }
}
