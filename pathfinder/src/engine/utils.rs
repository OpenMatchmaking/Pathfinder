/// Utility module for handling data in Open Matchmaking project.
///
use tungstenite::protocol::Message;

use json::object;

use crate::error::Result;
use crate::engine::serializer::{JsonMessage, Serializer};

/// Transforms an error (which is a string) into JSON object in the special format.
pub fn wrap_a_string_error(error_type: &str, err: &str) -> Message {
    let json_error_message = object!("type" => error_type, "details" => err);
    let serializer = Serializer::new();
    serializer.serialize(json_error_message.dump()).unwrap()
}

/// Serialize a JSON object into message.
pub fn serialize_message(json: JsonMessage) -> Message {
    let serializer = Serializer::new();
    serializer.serialize(json.dump()).unwrap()
}

/// Deserialize a message into JSON object.
pub fn deserialize_message(message: &Message) -> Result<JsonMessage> {
    let serializer = Serializer::new();
    serializer.deserialize(message)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use json::{object, parse as json_parse};
    use tungstenite::Message;

    use crate::engine::utils::{deserialize_message, serialize_message, wrap_a_string_error};

    #[test]
    fn test_wrap_an_string_error_returns_json_with_details_field() {
        let error_string = "some error";
        let dictionary = object!{"type" => "test", "details" => error_string};
        let expected = Message::Text(dictionary.dump());
        let result = wrap_a_string_error("test", error_string);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_serialize_message_returns_a_message_struct() {
        let dictionary = object!{"test" => "value"};
        let test_string = dictionary.dump();
        let raw_data = Arc::new(Box::new(json_parse(&test_string).unwrap()));
        let result = serialize_message(raw_data);

        assert_eq!(result.is_text(), true)
    }

    #[test]
    fn test_deserialize_message_returns_a_json_message() {
        let dictionary = object!{"url" => "test"};
        let message = Message::Text(dictionary.dump());
        let result = deserialize_message(&message);

        assert_eq!(result.is_ok(), true);
        let unwrapped_result = result.unwrap();
        assert_eq!(unwrapped_result.has_key("url"), true);
        assert_eq!(unwrapped_result["url"], dictionary["url"]);
    }

    #[test]
    fn test_deserialize_message_returns_an_error() {
        let invalid_json = String::from(r#"{"url": "test""#);
        let message = Message::Text(invalid_json);
        let result = deserialize_message(&message);

        assert_eq!(result.is_err(), true);
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Decoding error: Unexpected end of JSON"
        )
    }
}
