/*
 * This file contains the new error kind
 */

error_chain!{
    foreign_links {
        Io(::std::io::Error);
        Hyper(::hyper::Error);
        Url(::hyper::error::ParseError);
    }

    errors {
        PoisonError(e: String) {
            description(e)
            display("{}", e)
        }
        CannotParse {
            description("The url in this thread cannot be parse anymore")
            display("The url in this thread cannot be parse anymore")
        }
    }
}