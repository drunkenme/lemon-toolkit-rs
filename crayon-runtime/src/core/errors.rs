use graphics;
use super::window;
use resource;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Graphics(graphics::errors::Error, graphics::errors::ErrorKind);
        Window(window::Error, window::ErrorKind);
        Resource(resource::errors::Error, resource::errors::ErrorKind);
    }
}