
// example from rust book.

fn example_1() {
    let r;

    {
        let x = 42;
        r = &x;
    }

    println!("{}", *r);
}


// explains lifetimes like so:

fn example_1() {            //
    let r;                  // ----------+-- 'r
                            //           |
    {                       //           |
        let x = 42;         // --+-- 'x  |
        r = &x;             //   |       |
    }                       // --+       |
                            //           |
    println!("{}", *r);     //           |
}                           // ----------+


// but doesn't make sense,
// cause nothing changes,
// when remove the print,
// even though this is valid code.

fn example_1() {            //
    let r;                  // ----------+-- 'r
                            //           |
    {                       //           |
        let x = 42;         // --+-- 'x  |
        r = &x;             //   |       |
    }                       // --+       |
                            //           |
}                           // ----------+


// so this is in fact not what lifetimes are.
// lifetimes are regions of code,
// that a reference must be valid for.
// ie: assignment -> last use.
// aka: liveness.

fn example_1() {            //
    let r;                  // ----------+-- 'r
                            //           |
    {                       //           |
        let x = 42;         //           |
        r = &x;             //           |
    }                       //           |
                            //           |
    println!("{}", *r);     //           |
}                           // ----------+

fn example_1() {            //
    let r;                  //
                            //
    {                       //
        let x = 42;         //
        r = &x;             // ----------+-- 'r
    }                       //           |
                            //           |
    println!("{}", *r);     //           |
}                           // ----------+

fn example_1() {            //
    let r;                  //
                            //
    {                       //
        let x = 42;         //
        r = &x;             // ----------+-- 'r
    }                       //           |
                            //           |
    println!("{}", *r);     // ----------+
}                           //


// no use -> empty lifetime.

fn example_1() {            //
    let r;                  //
                            //
    {                       //
        let x = 42;         //
        r = &x;             // ------------- 'r
    }                       //
                            //
}                           //



// to illustrate this idea of liveness,
// here is another example.

fn example_2() {
    let foo = 69;
    let mut r;

    {
        let x = 42;
        r = &x;

        println!("{}", *r);
    }

    r = &foo;

    println!("{}", *r);
}


// and the lifetime.

fn example_2() {                //
    let foo = 69;               //
    let mut r;                  //
                                //
    {                           //
        let x = 42;             //
        r = &x;                 // ----------+-- 'r
                                //           |
        println!("{}", *r);     // ----------+
    }                           //
                                //
    r = &foo;                   // ----------+-- 'r
                                //           |
    println!("{}", *r);         // ----------+
}                               //


// but actually, knowing 'r isn't enough.
// we also need to know what r is referencing.

fn example_2() {                //
    let foo = 69;               //
    let mut r;                  //
                                //
    {                           //
        let x = 42;             //
        r = &x;                 // ----------+-- 'r = { &x }
                                //           |
        println!("{}", *r);     // ----------+
    }                           //
                                //
    r = &foo;                   // ----------+-- 'r = { &foo }
                                //           |
    println!("{}", *r);         // ----------+
}                               //


// now if we remove the reassignment:

fn example_2() {                //
    let foo = 69;               //
    let mut r;                  //
                                //
    {                           //
        let x = 42;             //
        r = &x;                 // ----------+-- 'r = { &x }
                                //           |
        println!("{}", *r);     //           |
    }                           //           |
                                //           |
    println!("{}", *r);         // ----------+
}                               //


// or if we made it conditional:

fn example_2() {                //
    let foo = 69;               //
    let mut r;                  //
                                //
    {                           //
        let x = 42;             //
        r = &x;                 // ----------+-- 'r = { &x }
                                //           |
        println!("{}", *r);     //           |
    }                           //           |
                                //           |
    if random_bool() {          //           |
        r = &foo;               //           +-- 'r = { &x, &foo }
    }                           //           |
                                //           |
    println!("{}", *r);         // ----------+
}                               //



// - separate liveness from regions.
// - introduce the set idea.



fn longest<'a>(
    s1: &'a str, 
    s2: &'a str)
    -> &'a str
{
    if s1.len() > s2.len() {
        return s1;
    }
    else {
        "hi"
        return s2;
    }
}



fn longest<'a>(s1: &'a str, s2: &'a str) -> &'a str {}


fn main() {
    let x: &'x str = "hi";
    let y: String  = "hello".into();
    let z: &'z str = "hey";

    let l1: &'l1 str = longest(x, &y); // 'y
    let l2: &'l2 str = longest(l1, z);
}



fn foo<'a, 'b>(a: &'a i32, b: &'b i32)
where 'a: 'b
{}



fn longest<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() {
        return s1;
    }
    else {
        return s2;
    }
}

fn longest<'s1, 's2, 'out>(s1: &'s1 str, s2: &'s2 str) -> &'out str {
    if s1.len() > s2.len() {
        return s1;
    }
    else {
        return s2;
    }
}

fn longest<'s1, 's2, 'out>(s1: &'s1 str, s2: &'s2 str) -> &'out str
    where 'x: 'y, ...
{
    if s1.len() > s2.len() {
        return s1;
    }
    else {
        return s2;
    }
}

fn longest<'s1, 's2, 'out>(s1: &'s1 str, s2: &'s2 str) -> &'out str
    where 's1: 'out, 's2: 'out
{
    if s1.len() > s2.len() {
        return s1;
    }
    else {
        return s2;
    }
}


    fn longest<'s1, 's2, 'out>(s1: &'s1 str, s2: &'s2 str) -> &'out str
        where 's1: 'out, 's2: 'out
{}
    fn longest<'a>(s1: &'a str, s2: &'a str) -> &'a str

{}

