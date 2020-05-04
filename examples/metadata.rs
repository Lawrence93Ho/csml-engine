use csmlinterpreter::data::csml_bot::CsmlBot;
use csmlinterpreter::data::csml_flow::CsmlFlow;
use csmlinterpreter::data::event::Event;
use csmlinterpreter::data::ContextJson;
use csmlinterpreter::interpret;

const DEFAULT_ID_NAME: &str = "id";
const DEFAULT_FLOW_NAME: &str = "default";
const DEFAULT_STEP_NAME: &str = "start";
const DEFAULT_BOT_NAME: &str = "my_bot";

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTION
////////////////////////////////////////////////////////////////////////////////

fn main() {
    let default_content = std::fs::read_to_string("CSML/examples/metadata.csml").unwrap();
    let default_flow = CsmlFlow::new(DEFAULT_ID_NAME, "default", &default_content, Vec::default());

    // Create a CsmlBot
    let bot = CsmlBot::new(
        DEFAULT_ID_NAME,
        DEFAULT_BOT_NAME,
        None,
        vec![default_flow],
        DEFAULT_FLOW_NAME,
    );

    // Create an Event
    let event = Event::default();

    // Create a Metadata
    let metadata = serde_json::json!({
        "firstname":"Toto",
        "email":"toto@clevy.io"
    });

    // Create context
    let context = ContextJson::new(
        serde_json::json!({}),
        metadata,
        None,
        None,
        DEFAULT_STEP_NAME,
        DEFAULT_FLOW_NAME,
    );

    // Run interpreter
    dbg!(interpret(bot, context, event, None));
}
