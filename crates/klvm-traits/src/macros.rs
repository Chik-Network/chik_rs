/// Converts a list of KLVM values into a series of nested pairs.
#[macro_export]
macro_rules! klvm_list {
    () => {
        ()
    };
    ( $first:expr $( , $rest:expr )* $(,)? ) => {
        ($first, $crate::klvm_list!( $( $rest ),* ))
    };
}

/// Converts a tuple of KLVM values into a series of nested pairs.
#[macro_export]
macro_rules! klvm_tuple {
    () => {
        ()
    };
    ( $first:expr $(,)? ) => {
        $first
    };
    ( $first:expr $( , $rest:expr )* $(,)? ) => {
        ($first, $crate::klvm_tuple!( $( $rest ),* ))
    };
}

/// Quotes a KLVM value.
#[macro_export]
macro_rules! klvm_quote {
    ( $value:expr ) => {
        (1, $value)
    };
}

/// Constructs a sequence of nested pairs that represents a set of curried arguments.
#[macro_export]
macro_rules! klvm_curried_args {
    () => {
        1
    };
    ( $first:expr $( , $rest:expr )* $(,)? ) => {
        (4, ($crate::klvm_quote!($first), ($crate::klvm_curried_args!( $( $rest ),* ), ())))
    };
}

/// Creates the type needed to represent a list of KLVM types.
#[macro_export]
macro_rules! match_list {
    () => {
        ()
    };
    ( $first:ty $( , $rest:ty )* $(,)? ) => {
        ($first, $crate::match_list!( $( $rest ),* ))
    };
}

/// Creates the type needed to represent a tuple of KLVM types.
#[macro_export]
macro_rules! match_tuple {
    () => {
        ()
    };
    ( $first:ty $(,)? ) => {
        $first
    };
    ( $first:ty $( , $rest:ty )* $(,)? ) => {
        ($first, $crate::match_tuple!( $( $rest ),* ))
    };
}

/// Creates the type needed to represent a quoted KLVM type.
#[macro_export]
macro_rules! match_quote {
    ( $type:ty ) => {
        ($crate::MatchByte::<1>, $type)
    };
}

/// Creates the type needed to represent a set of curried arguments.
#[macro_export]
macro_rules! match_curried_args {
    () => {
        $crate::MatchByte::<1>
    };
    ( $first:ty $( , $rest:ty )* $(,)? ) => {
        (
            $crate::MatchByte::<4>,
            (
                $crate::match_quote!($first),
                ($crate::match_curried_args!( $( $rest ),* ), ()),
            ),
        )
    };
}

/// Deconstructs a KLVM list that has been matched.
#[macro_export]
macro_rules! destructure_list {
    () => {
        _
    };
    ( $first:pat $( , $rest:pat )* $(,)? ) => {
        ($first, $crate::destructure_list!( $( $rest ),* ))
    };
}

/// Deconstructs a KLVM tuple that has been matched.
#[macro_export]
macro_rules! destructure_tuple {
    () => {
        _
    };
    ( $first:pat $(,)? ) => {
        $first
    };
    ( $first:pat $( , $rest:pat )* $(,)? ) => {
        ($first, $crate::destructure_tuple!( $( $rest ),* ))
    };
}

/// Deconstructs a quoted KLVM value that has been matched.
#[macro_export]
macro_rules! destructure_quote {
    ( $name:pat ) => {
        (_, $name)
    };
}

/// Deconstructs a set of curried arguments that has been matched.
#[macro_export]
macro_rules! destructure_curried_args {
    () => {
        _
    };
    ( $first:pat $( , $rest:pat )* $(,)? ) => {
        (
            _,
            (
                $crate::destructure_quote!($first),
                ($crate::destructure_curried_args!( $( $rest ),* ), ()),
            ),
        )
    };
}
