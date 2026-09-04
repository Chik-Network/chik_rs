/// Converts a list of CLVK values into a series of nested pairs.
#[macro_export]
macro_rules! clvk_list {
    () => {
        ()
    };
    ( $first:expr $( , $rest:expr )* $(,)? ) => {
        ($first, $crate::clvk_list!( $( $rest ),* ))
    };
}

/// Converts a tuple of CLVK values into a series of nested pairs.
#[macro_export]
macro_rules! clvk_tuple {
    () => {
        ()
    };
    ( $first:expr $(,)? ) => {
        $first
    };
    ( $first:expr $( , $rest:expr )* $(,)? ) => {
        ($first, $crate::clvk_tuple!( $( $rest ),* ))
    };
}

/// Quotes a CLVK value.
#[macro_export]
macro_rules! clvk_quote {
    ( $value:expr ) => {
        (1, $value)
    };
}

/// Constructs a sequence of nested pairs that represents a set of curried arguments.
#[macro_export]
macro_rules! clvk_curried_args {
    () => {
        1
    };
    ( $first:expr $( , $rest:expr )* $(,)? ) => {
        (4, ($crate::clvk_quote!($first), ($crate::clvk_curried_args!( $( $rest ),* ), ())))
    };
}

/// Creates the type needed to represent a list of CLVK types.
#[macro_export]
macro_rules! match_list {
    () => {
        ()
    };
    ( $first:ty $( , $rest:ty )* $(,)? ) => {
        ($first, $crate::match_list!( $( $rest ),* ))
    };
}

/// Creates the type needed to represent a tuple of CLVK types.
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

/// Creates the type needed to represent a quoted CLVK type.
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

/// Deconstructs a CLVK list that has been matched.
#[macro_export]
macro_rules! destructure_list {
    () => {
        _
    };
    ( $first:pat $( , $rest:pat )* $(,)? ) => {
        ($first, $crate::destructure_list!( $( $rest ),* ))
    };
}

/// Deconstructs a CLVK tuple that has been matched.
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

/// Deconstructs a quoted CLVK value that has been matched.
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
