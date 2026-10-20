use zynx::models::{gateway::*, payloads::*, rest::*, voice::*};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

fn decode<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

fn variant<T: DeserializeOwned + Serialize + std::fmt::Debug>(value: Value, expected: &str) {
    let parsed: T = decode(value.clone());
    assert!(format!("{parsed:?}").starts_with(expected), "{parsed:?}");
    let serialized = serde_json::to_value(&parsed).unwrap();
    let again: T = decode(serialized);
    assert!(format!("{again:?}").starts_with(expected));
}

#[test]
fn gateway_control_packets_use_numeric_opcodes() {
    for (op, data, expected) in [
        (1, Value::Null, "GatewayHeartbeatRequest"),
        (7, Value::Null, "GatewayReconnect"),
        (9, json!(false), "GatewayInvalidSession"),
        (10, json!({"heartbeat_interval": 45000}), "GatewayHello"),
        (11, Value::Null, "GatewayHeartbeatAck"),
    ] {
        variant::<GatewayReceivePayload>(json!({"op":op,"d":data,"s":null,"t":null}), expected);
    }
    variant::<GatewaySendPayload>(json!({"op":1,"d":42}), "GatewayHeartbeat");
    assert!(serde_json::from_value::<GatewayReceivePayload>(json!({"op":99,"d":null})).is_err());
    assert!(serde_json::from_value::<GatewayReceivePayload>(json!({"op":10,"d":null})).is_err());
}

#[test]
fn dispatch_uses_event_name_even_when_data_shapes_match() {
    for (event, expected) in [
        ("GUILD_DELETE", "GatewayGuildDeleteDispatch"),
        (
            "GUILD_INTEGRATIONS_UPDATE",
            "GatewayGuildIntegrationsUpdateDispatch",
        ),
        ("WEBHOOKS_UPDATE", "GatewayWebhooksUpdateDispatch"),
    ] {
        variant::<GatewayDispatchPayload>(
            json!({"op":0,"s":42,"t":event,"d":{"id":"1","guild_id":"2","channel_id":"3"}}),
            expected,
        );
    }
    variant::<GatewayDispatchPayload>(
        json!({"op":0,"s":42,"t":"RESUMED","d":{}}),
        "GatewayResumedDispatch",
    );
    assert!(
        serde_json::from_value::<GatewayDispatchPayload>(
            json!({"op":1,"s":42,"t":"RESUMED","d":null})
        )
        .is_err()
    );
}

#[test]
fn sparse_message_updates_preserve_missing_empty_and_null() {
    let value = json!({"id":"1","channel_id":"2","content":"","edited_timestamp":null,"referenced_message":null});
    let update: GatewayMessageUpdateDispatchData = decode(value.clone());
    assert_eq!(update.message.content.as_deref(), Some(""));
    assert!(update.message.author.is_none());
    assert!(update.extra.mentions.is_none());
    assert_eq!(update.message.edited_timestamp, Some(None));
    assert_eq!(update.message.referenced_message, Some(None));
    assert_eq!(serde_json::to_value(update).unwrap(), value);
    variant::<GatewayDispatchPayload>(
        json!({"op":0,"t":"MESSAGE_UPDATE","s":1,"d":{"id":"1","channel_id":"2","embeds":[]}}),
        "GatewayMessageUpdateDispatch",
    );
    assert!(
        serde_json::from_value::<GatewayMessageUpdateDispatchData>(json!({"content":"x"})).is_err()
    );
}

#[test]
fn messages_decode_role_ids_and_numeric_flags() {
    let mut value: Value =
        serde_json::from_str(include_str!("fixtures/message_create.json")).unwrap();
    value["mention_roles"] = json!(["123"]);
    value["flags"] = json!(1u64 << 60);
    let message: GatewayMessageCreateDispatchData = decode(value);
    assert_eq!(message.message.mention_roles, ["123"]);
    assert_eq!(message.message.flags.unwrap().bits(), 1u64 << 60);
    assert_eq!(
        serde_json::to_value(message).unwrap()["flags"],
        json!(1u64 << 60)
    );
}

#[test]
fn responses_with_identical_shapes_use_response_type() {
    for (kind, expected) in [
        (1, "APIInteractionResponsePong"),
        (5, "APIInteractionResponseDeferredChannelMessageWithSource"),
        (6, "APIInteractionResponseDeferredMessageUpdate"),
        (7, "APIInteractionResponseUpdateMessage"),
        (10, "APIPremiumRequiredInteractionResponse"),
        (12, "APIInteractionResponseLaunchActivity"),
    ] {
        variant::<APIInteractionResponse>(json!({"type":kind}), expected);
    }
    assert!(serde_json::from_value::<APIInteractionResponse>(json!({"type":99})).is_err());
}

#[test]
fn ping_interactions_and_webhooks_accept_their_actual_minimal_envelopes() {
    variant::<APIInteraction>(
        json!({"id":"1","application_id":"2","type":1,"token":"test","version":1}),
        "APIPingInteraction",
    );
    variant::<APIWebhookEvent>(json!({"version":1,"application_id":"2","type":0}), "Ping");
}

#[test]
fn select_components_do_not_all_become_channel_selects() {
    for (kind, expected) in [
        (3, "APIStringSelectComponent"),
        (5, "APIUserSelectComponent"),
        (6, "APIRoleSelectComponent"),
        (7, "APIMentionableSelectComponent"),
        (8, "APIChannelSelectComponent"),
    ] {
        variant::<APISelectMenuComponent>(
            json!({"type":kind,"custom_id":"selection","options":[]}),
            expected,
        );
    }
    variant::<APIMessageComponentInteractionData>(
        json!({"component_type":3,"custom_id":"selection","values":["a"]}),
        "APIMessageSelectMenuInteractionData",
    );
}

#[test]
fn component_shapes_cannot_override_type_or_button_style() {
    variant::<APIMessageComponent>(
        json!({"type":10,"content":"text"}),
        "APITextDisplayComponent",
    );
    variant::<APIMessageComponent>(
        json!({"type":14,"divider":true,"spacing":1}),
        "APISeparatorComponent",
    );
    variant::<APIButtonComponent>(
        json!({"type":2,"style":5,"url":"https://example.com","custom_id":"ignored"}),
        "APIButtonComponentWithURL",
    );
    assert!(
        serde_json::from_value::<APIMessageComponent>(json!({"type":99,"content":"text"})).is_err()
    );
    assert!(
        serde_json::from_value::<APIButtonComponent>(
            json!({"type":2,"style":5,"custom_id":"not-a-link"})
        )
        .is_err()
    );
}

#[test]
fn command_option_types_preserve_distinct_snowflake_variants() {
    for (kind, expected) in [
        (3, "String"),
        (6, "User"),
        (7, "Channel"),
        (8, "Role"),
        (9, "Mentionable"),
        (11, "Attachment"),
    ] {
        variant::<APIApplicationCommandInteractionDataBasicOption>(
            json!({"type":kind,"name":"option","value":"123"}),
            expected,
        );
        variant::<APIApplicationCommandBasicOption>(
            json!({"type":kind,"name":"option","description":"description"}),
            expected,
        );
    }
    variant::<APIApplicationCommandInteractionDataOption>(
        json!({"type":1,"name":"sub","options":[{"type":6,"name":"user","value":"123"}]}),
        "Subcommand",
    );
    assert!(
        serde_json::from_value::<APIApplicationCommandInteractionDataBasicOption>(
            json!({"type":5,"name":"boolean","value":"false"})
        )
        .is_err()
    );
}

#[test]
fn rest_command_type_wins_over_extra_description() {
    variant::<RESTPostAPIApplicationCommandsJSONBody>(
        json!({"type":4,"name":"launch","description":"activity"}),
        "PrimaryEntryPoint",
    );
    variant::<RESTPostAPIApplicationCommandsJSONBody>(
        json!({"type":2,"name":"user","description":"extra"}),
        "ContextMenu",
    );
    variant::<RESTPostAPIApplicationCommandsJSONBody>(
        json!({"name":"command","description":"description"}),
        "ChatInput",
    );
}

#[test]
fn resolved_threads_keep_their_thread_fields() {
    variant::<APIInteractionDataResolvedChannel>(
        json!({"id":"1","type":11,"permissions":"0","parent_id":"2","thread_metadata":{"archived":false}}),
        "Thread",
    );
}

#[test]
fn flags_support_combinations_and_optional_guild_member_flags() {
    let speaking: VoiceSpeakingFlags = decode(json!(3));
    assert!(speaking.contains(VoiceSpeakingFlags::Microphone | VoiceSpeakingFlags::Soundshare));
    assert_eq!(serde_json::to_value(speaking).unwrap(), json!(3));
    let user: UserFlags = decode(json!(3));
    assert!(user.contains(UserFlags::Staff | UserFlags::Partner));
    let _: APIFlaggedGuildMember = decode(json!({}));
    let flags: APIFlaggedGuildMember = decode(json!({"flags":1u32 << 30}));
    assert_eq!(
        serde_json::to_value(flags).unwrap()["flags"],
        json!(1u32 << 30)
    );
}

#[test]
fn nullable_rest_flags_keep_all_three_states() {
    for value in [json!({}), json!({"flags":null}), json!({"flags":4})] {
        let body: RESTPatchAPIChannelMessageJSONBody = decode(value.clone());
        let encoded = serde_json::to_value(body).unwrap();
        assert_eq!(encoded.get("flags"), value.get("flags"));
    }
}

#[test]
fn voice_packets_use_numeric_opcode_and_combined_speaking_flags() {
    variant::<VoiceReceivePayload>(json!({"op":9,"d":{}}), "VoiceResumed");
    variant::<VoiceReceivePayload>(
        json!({"op":8,"d":{"v":8,"heartbeat_interval":45000}}),
        "VoiceHello",
    );
    variant::<VoiceReceivePayload>(
        json!({"op":5,"d":{"user_id":"1","ssrc":2,"speaking":3}}),
        "VoiceSpeaking",
    );
}

#[test]
fn scheduled_events_use_entity_type_for_identical_channel_event_shapes() {
    for (kind, expected) in [
        (1, "APIStageInstanceGuildScheduledEvent"),
        (2, "APIVoiceGuildScheduledEvent"),
        (3, "APIExternalGuildScheduledEvent"),
    ] {
        variant::<APIGuildScheduledEvent>(
            json!({
                "id":"1","guild_id":"2","channel_id":if kind==3 {Value::Null} else {json!("3")},
                "name":"event","scheduled_start_time":"2026-10-20T12:00:00Z",
                "scheduled_end_time":"2026-10-20T13:00:00Z","privacy_level":2,
                "status":1,"entity_type":kind,"entity_metadata":if kind==3 {json!({"location":"outside"})} else {Value::Null}
            }),
            expected,
        );
    }
}

#[test]
fn guild_interactions_decode_the_member_and_command_kind() {
    let interaction = json!({
        "id":"1","application_id":"2","type":2,"token":"test","version":1,
        "guild_id":"3","app_permissions":"0","locale":"en-US","entitlements":[],
        "authorizing_integration_owners":{"0":"3"},"attachment_size_limit":10485760,
        "member":{"user":{"id":"4","username":"user","discriminator":"0"},
            "roles":[],"joined_at":"2026-10-19T12:00:00Z","deaf":false,"mute":false,"permissions":"0"},
        "data":{"id":"5","name":"launch","type":4}
    });
    variant::<APIInteraction>(interaction.clone(), "APIApplicationCommandInteraction");
    variant::<APIApplicationCommandInteraction>(
        interaction,
        "APIPrimaryEntryPointCommandInteraction",
    );
}

#[test]
fn incomplete_oauth_credentials_do_not_silently_become_absent() {
    for value in [
        json!({"client_id":"1"}),
        json!({"client_secret":"test"}),
        json!({"client_id":1,"client_secret":"test"}),
    ] {
        assert!(serde_json::from_value::<RESTOAuth2TokenOptionalClientCredentials>(value).is_err());
    }
    variant::<RESTOAuth2TokenOptionalClientCredentials>(json!({}), "Without");
    variant::<RESTOAuth2TokenOptionalClientCredentials>(
        json!({"client_id":"1","client_secret":"test"}),
        "With",
    );
}
