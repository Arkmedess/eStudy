//! Testes de contrato para identificadores `UUIDv7`, relógio virtual e barramento de eventos.

#![allow(clippy::expect_used)]

use chrono::{DateTime, Duration};
use estudy_domain::{
    ClockPort, DomainError, DomainEvent, EntityId, EventBusPort, FixedClock, InMemoryEventBus,
};

#[test]
fn should_parse_and_validate_entity_ids() {
    let first = EntityId::new_v7();
    let reparsed = EntityId::parse(&first.to_string()).expect("UUID válido deve fazer parse");
    assert_eq!(first, reparsed);

    let err = EntityId::parse("not-a-uuid").expect_err("texto inválido deve retornar erro");
    assert!(matches!(err, DomainError::InvalidEntityId(_)));
}

#[test]
fn should_advance_fixed_clock_deterministically() {
    let start = DateTime::from_timestamp(1_760_000_000, 0).expect("timestamp válido");
    let clock = FixedClock::new(start);
    assert_eq!(clock.now_utc(), start);

    clock.advance(Duration::minutes(25));
    assert_eq!(clock.now_utc(), start + Duration::minutes(25));
}

#[test]
fn should_record_published_domain_events_in_order() {
    let bus = InMemoryEventBus::new();
    let profile_id = EntityId::new_v7();

    bus.publish(DomainEvent::DatabaseMigrated { version: 1 });
    bus.publish(DomainEvent::WalletEntryRecorded {
        profile_id: profile_id.clone(),
        seq: 1,
        delta: 20,
    });

    let events = bus.snapshot();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], DomainEvent::DatabaseMigrated { version: 1 });
    assert_eq!(
        events[1],
        DomainEvent::WalletEntryRecorded {
            profile_id,
            seq: 1,
            delta: 20,
        }
    );
}
