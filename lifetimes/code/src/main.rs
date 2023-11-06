// introduce lifetime vs region.
// and liveness.
fn example_1() {
    let r;

    {
        let x = 42;
        r = &x;
    }

    //println!("{}", *r);
}


// assignment resets borrows.
// expand upon liveness.
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


// control flow & regions as sets.
fn example_3() {
    let a = "hello".to_string();
    let b = "hi".to_string();

    let r =
        if a.len() > b.len() { &a }
        else                 { &b };

    //drop(a);

    println!("{}", r);
}


fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a }
    else                 { b }
}

// abstraction.
fn example_4() {
    let a = "hello".to_string();
    let b = "hi".to_string();

    let r = longest(&a, &b);

    //drop(a);

    println!("{}", r);
}


// independent sets.
fn swap<'a, 'b>(a: &'a str, b: &'b str) -> (&'b str, &'a str) {
    (b, a)
}

fn example_5() {
    let a = "hello".to_string();
    let b = "hi".to_string();

    let (rb, _ra) = swap(&a, &b);

    drop(a);

    println!("{}", rb);
}


// subsets: swap with `(a, b)`.
// and interpreting the error messages.


fn main() {
    example_1();
    example_2();
    example_3();
    example_4();
    example_5();
}


fn longest2<'s1, 's2, 'out>(s1: &'s1 str, s2: &'s2 str) -> &'out str
    where 's1: 'out, 's2: 'out
{
    if s1.len() > s2.len() {
        return s1;
    }
    else {
        return s2;
    }
}

