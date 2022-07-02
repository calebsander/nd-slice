# N-dimensional slices

In this post, I'll introduce the idea of an "N-dimensional slice" and explore different implementations in Rust.
I hope to convince you that the "stride vector" approach is especially elegant and allows many common operations to avoid copying data.
We'll start with matrices, the simplest case, and then see how the ideas generalize to any number of dimensions.

## What is an N-dimensional slice?

Okay, before I start talking about "N-dimensional slices", I should probably give a definition.
Let's start by considering a single possible value for N: 1.
A 1-dimensional slice is a standard array or list.
Each element has an index (consisting of **1** number).
In Rust, we would represent this with the type `&[T]` (or `&mut [T]`, `Box<[T]>`, etc. depending on the ownership).
<details>
  <summary>If you're unfamiliar with "boxed slices" in Rust</summary>

  Don't worry, I think `Box<[T]>` is underused in Rust code!
  It's similar to `Vec<T>`: it's an owned, heap-allocated slice of `T`s.
  The difference is that it's created with a fixed length, so it can't be grown.
  In exchange for this restriction, it has the advantage of only consisting of a pointer and a length, whereas a `Vec<T>` stores a pointer, a length, *and a capacity*.
  So it shaves a full 1/3 off the size of a `Vec<T>` (and potentially more off the heap space for the elements, since no extra is allocated)!

  `Box<[T]>` is actually not a special type; it is just a combination of `Box` and `[T]`.
  `[T]` (a slice of `T`s) is a valid type in Rust.
  But since it's not `Sized`, it can't be passed around by value.
  It needs to be behind some kind of pointer.
  This can be `&`, `&mut`, `Box` or even more exotically, `Rc`, `Arc`, or `Cow`.
  `Rc<[T]>`, a reference-counted slice of `T`s, has no `Vec` equivalent!
</details>

Here's a Rust example of a 1-dimensional slice:
```rust
fn main() {
  // A 1-dimensional array of chars with length 3
  let data: [char; 3] = ['A', 'B', 'C'];
  // A 1-dimensional slice (borrowed immutably) with length 3
  let slice: &[char] = &data;
  // The slice's length is 1 number
  println!("Length: {}", slice.len());
  // We can access elements of the slice by indexing with 1 number
  println!("Elements: {:?}, {:?}, {:?}", slice[0], slice[1], slice[2]);
}
```
This prints:
```
Length: 3
Elements: 'A', 'B', 'C'
```

In a 2-dimensional slice (usually called a "matrix"), each element's index consists of **2** indices, one for each dimension.
Similarly, the length consists of 2 numbers, the length in each dimension.
Rust doesn't have builtin support for 2-dimensional slices, but it does allow for multi-dimensional arrays, as long as the length of each dimension is known when the program is compiled.
For example:
```rust
fn main() {
  // A 2-dimensional array of chars with length (2, 3). That may look backwards,
  // but 3 is the length of the inner arrays, so it is the second dimension.
  let data: [[char; 3]; 2] = [
    ['A', 'B', 'C'],
    ['D', 'E', 'F'],
  ];
  // Each element of `data` has a unique index (i, j),
  // where `i` and `j` range from 0 to the length in each dimension
  for i in 0..2 {
    for j in 0..3 {
      println!("data[{}][{}] = {:?}", i, j, data[i][j]);
    }
  }
}
```
This prints:
```
data[0][0] = 'A'
data[0][1] = 'B'
data[0][2] = 'C'
data[1][0] = 'D'
data[1][1] = 'E'
data[1][2] = 'F'
```

How about a 0-dimensional slice?
<details>
  <summary>No, seriously, think about it! OK, here's the answer:</summary>

  Its length is 0-dimensional, i.e. `()`.
  So you might assume the slice would have 0 elements.
  But in fact, that's not the case...

  Each index in the slice has an index consists of 0 numbers, i.e. it has the form `()`.
  How many values are there of type `()`?
  Just the one!
  So a 0-dimensional slice contains just a single value.
  For example, a 0-dimensional slice of `i32`s is just an `i32`.

  Another way to see this is to notice that an N-dimensional slice with length `(length_1, length_2, ..., length_N)` has `length_1 * length_2 * ... * length_N` total elements.
  So the number of elements in a slice with length `()` is the empty product, 1.
  Rust confirms:
  ```rust
  fn main() {
    let empty: [usize; 0] = [];
    println!("The empty product is: {}", empty.into_iter().product::<usize>());
  }
  ```
  Prints:
  ```
  The empty product is: 1
  ```
</details>

As with 1-dimensional slices, we will require that every value in an N-dimensional slice has the same type.

## But why?

So a 1-dimensional slice is a normal slice and a 0-dimensional slice is just a value.
Maybe you accept those are useful, but why would we need more dimensions?

Well, lots of datasets are naturally multi-dimensional.
For example, if we measure the same data points across different samples, or at different points in time, we will have a 2-dimensional dataset.
Each value can be indexed by `(measurement #, sample #)` or `(measurement #, time #)`.

And if there were data for more combinations of inputs, we could end up with higher-dimensional slices.
For example, a dataset consisting of the grade for each student in each class each year would be 3-dimensional.

See [the end of the post](#an-example) for an example of how we'll be able to manipulate N-dimensional slices with our final code.

## Representing a 2-dimensional slice, first try

We saw above that if we can know the length of our 2-dimensional slice at compile time, we can represent it with a nested array.
But often, our slice's length will be determined during the execution of our program.
And we want to write code to operate on N-dimensional slices regardless of their lengths.

Rust already has 0-dimensional slices (`T`) and 1-dimensional slices (`&[T]`, `Box<[T]>`, etc.).
It looks like the pattern is to just slap another `Box<[` and `]>` around the type and you get 1 more dimension.
Let's try it out:
```rust
/// A 2-dimensional (owned) slice of i32 elements
struct Slice2D(Box<[Box<[i32]>]>);

impl Slice2D {
  /// Makes a length0 x length1 2-dimensional slice of 0s
  fn zero(length0: usize, length1: usize) -> Self {
    // For each index in the 0th dimension, make a row
    let rows = (0..length0).map(|_| {
      // For each index in the 1st dimension, add a 0 to the row
      (0..length1).map(|_| 0).collect()
    }).collect();
    Self(rows)
  }

  /// Gets the 2-dimensional length of the slice
  fn len(&self) -> (usize, usize) {
    // Question: for what lengths does this not work?
    (self.0.len(), self.0[0].len())
  }

  /// Gets the value of the element at a given index
  fn get(&self, index0: usize, index1: usize) -> i32 {
    self.0[index0][index1]
  }

  /// Sets the value of the element at a given index
  fn set(&mut self, index0: usize, index1: usize, value: i32) {
    self.0[index0][index1] = value;
  }
}
```
`len()`, `get()`, and `set()` are the core operations we need.
Some way to make a new slice is also necessary: we can use `zero()`, followed by `set()` if we want the elements not to be 0.

With these functions, we can implement whatever complex processing of 2-dimensional slices we want.
For example:
```rust
/// Makes a new slice by adding `value` to each element of `slice`
fn add_to_all_elements(slice: &Slice2D, value: i32) -> Slice2D {
  let (length0, length1) = slice.len();
  let mut new_slice = Slice2D::zero(length0, length1);
  for index0 in 0..length0 {
    for index1 in 0..length1 {
      new_slice.set(index0, index1, slice.get(index0, index1) + value);
    }
  }
  new_slice
}

/// Prints the contents of a slice for debugging purposes
fn print_slice(slice: &Slice2D) {
  let (length0, length1) = slice.len();
  for index0 in 0..length0 {
    for index1 in 0..length1 {
      print!("{} ", slice.get(index0, index1));
    }
    println!();
  }
}

fn main() {
  let mut slice = Slice2D::zero(3, 3);
  slice.set(0, 0, 1); slice.set(0, 1, 2); slice.set(0, 2, 3);
  slice.set(1, 0, 8); slice.set(1, 1, 9); slice.set(1, 2, 4);
  slice.set(2, 0, 7); slice.set(2, 1, 6); slice.set(2, 2, 5);
  println!("Original slice:");
  print_slice(&slice);
  let slice = add_to_all_elements(&slice, 10);
  println!("New slice:");
  print_slice(&slice);
}
```
This prints the original slice we created and the new one where 10 has been added to all the elements:
```
Original slice:
1 2 3
8 9 4
7 6 5
New slice:
11 12 13
18 19 14
17 16 15
```

### Pros and cons

One thing this approach definitely has going for it is how little code was required.
An added feature we get is being able to extract a single row (all elements with a particular 0th index) by partially indexing it:
```rust
impl Slice2D {
  /// Gets a 1-dimensional slice containing
  /// all elements with a particular 0th index
  fn extract_row(&self, index0: usize) -> &[i32] {
    &self.0[index0]
  }
}

fn main() {
  // ...
  println!("Original slice:");
  print_slice(&slice);
  println!("Row 1: {:?}", slice.extract_row(1));
}
```
This prints:
```
Original slice:
1 2 3
8 9 4
7 6 5
Row 1: [8, 9, 4]
```
Unfortunately, we can't do the same for the 1st index.

A major downside is the memory-efficiency of this representation.
We store each row in a separate `Box<i32>`.
To get a sense of the overhead, imagine we have a 10 x 10 slice of `i32`s.
This requires `10 * 10 * size_of::<i32>() = 400` bytes to store the elements.

We then have 10 rows represented as `Box<[i32]>`s, each of which requires 16 bytes (a pointer and a length).
So that's another 160 bytes of overhead.
(Part of this is due to a lot of just pointing elsewhere, and part is due to all rows having the same length, so storing it 10 times is redundant.)
And the outer `Box<[Box<[i32]>]>` takes another 16 bytes.
176 bytes of total overhead is almost half the space we need for the elements themselves.
We also have 11 separate heap allocations (the 10 rows plus the slice of rows), and if you know how memory allocators work, extra space is likely needed for book-keeping and alignment of each allocation.
That's not great.

## Representing a 2-dimensional slice, second try

Our main complaint with the previous representation was its memory overhead, so let's fix that!
The key idea is to avoid a separate allocation for each row.
Instead, we'll "flatten" the 2-dimensional slice into a 1-dimensional allocation.

Consider this 2x3 2-dimensional slice:
```
4 3 1
2 6 5
```
There are two ways we might reasonable flatten it into a single 1-dimensional slice with 6 elements, generally called ["row-major" and "column-major" orders](https://en.wikipedia.org/wiki/Row-_and_column-major_order):
- In row-major order, we list each row in order, so the elements within each row are contiguous.
  This looks like `[4, 3, 1, 2, 6, 5]` for our sample slice.
- In column-major order, we list each *column* in order, so the elements within each *column* are contiguous.
  This looks like `[4, 2, 3, 6, 1, 5]` for our sample slice.

Either approach works, and there is no clear reason to prefer one to the other.
The only difference is in how we find an element in the 1-dimensional slice:
- In row-major order, going down one row (incrementing `index0`) advances 3 (`length1`) elements in the slice.
  Going right one column (incrementing `index1`) advances 1 element in the slice.
- In column-major order, going down one row (incrementing `index0`) advances 1 element in the slice.
  Going right one column (incrementing `index1`) advances 2 (`length0`) elements in the slice.

We can see this in action:
```rust
fn main() {
  let length0 = 2usize;
  let length1 = 3;
  let row_major_slice = [4, 3, 1, 2, 6, 5];
  let row_major_index = |index0, index1| index0 * length1 + index1;
  for index0 in 0..length0 {
    for index1 in 0..length1 {
      print!("{} ", row_major_slice[row_major_index(index0, index1)]);
    }
    println!();
  }
  println!();

  let column_major_slice = [4, 2, 3, 6, 1, 5];
  let column_major_index = |index0, index1| index0 + index1 * length0;
  for index0 in 0..length0 {
    for index1 in 0..length1 {
      print!("{} ", column_major_slice[column_major_index(index0, index1)]);
    }
    println!();
  }
}
```
Both approaches print the same 2-dimensional slice:
```
4 3 1
2 6 5

4 3 1
2 6 5
```

We'll make the arbitrary choice to use row-major for now.
Here's what our new type looks like:
```rust
struct Slice2D {
  data: Box<[i32]>,
  length0: usize,
  length1: usize,
}
```
`data` stores the flattened 1-dimensional slice in row-major order.
(`data` consists of both the pointer to the first element and the length of the flattened slice.
We actually only need the pointer, since the length can be computed from `length0 * length1`.
But making this optimization requires some `unsafe` code, so we'll leave it for later.)

Consider the size of our 10 x 10 slice of `i32`s under this representation: `data` will be a 400-byte allocation, then we need `16 + 8 + 8 = 32` more bytes for the fields of the struct (can be reduced to 24 if we make `data` just a `*mut i32`).
32 or 24 bytes of overhead for the 400 bytes of data is way better than the 176 bytes we used previously!

Here's what the methods look like now:
```rust
impl Slice2D {
  fn zero(length0: usize, length1: usize) -> Self {
    let data = vec![0; length0 * length1].into_boxed_slice();
    Self { data, length0, length1 }
  }

  fn len(&self) -> (usize, usize) {
    (self.length0, self.length1)
  }

  /// Helper method for computing a row-major index
  fn index(&self, index0: usize, index1: usize) -> usize {
    index0 * self.length1 + index1
  }

  fn get(&self, index0: usize, index1: usize) -> i32 {
    self.data[self.index(index0, index1)]
  }

  fn set(&mut self, index0: usize, index1: usize, value: i32) {
    self.data[self.index(index0, index1)] = value;
  }
}
```
We can operate on a sample slice just as we did before:
```
Original slice:
1 2 3
8 9 4
7 6 5
New slice:
11 12 13
18 19 14
17 16 15
```

And since the slice is flattened in row-major order, we can still extract individual rows.
(If it were in column-major order, we could extract columns instead.)
```rust
impl Slice2D {
  fn extract_row(&self, index0: usize) -> &[i32] {
    // The row starts at index `(index0, 0)` and is `length1` elements long
    &self.data[self.index(index0, 0)..][..self.length1]
  }
}
```
Our row extraction program works just as before:
```
Original slice:
1 2 3
8 9 4
7 6 5
Row 1: [8, 9, 4]
```

## Representing a 2-dimensional slice, third try

Here's an interesting question: what happens if we *reinterpret* a row-major slice as a column-major slice?
Well, then advancing one element in the slice (which used to mean going right one column) now means going down one row.
So the meanings of the row and column indices are swapped!
This means we also need to swap `length0` and `length1`, since these tell us the number of rows and columns.
Let's try it:
```rust
fn main() {
  // Lengths swapped
  let length0 = 3usize;
  let length1 = 2;
  // ...
  for index0 in 0..length0 {
    for index1 in 0..length1 {
      // Access the row-major slice as though it were a column-major slice
      print!("{} ", row_major_slice[column_major_index(index0, index1)]);
    }
    println!();
  }
}
```
Recall the original slice was:
```
4 3 1
2 6 5
```
Our new program prints:
```
4 2
3 6
1 5
```
If you've seen some linear algebra, you may recognize this as the matrix "transpose" operation, which reflects the elements across the diagonal (4 and 6 stay in place, 2 and 3 swap, etc.).
(And this isn't just true for our sample matrix: switching between row-major and column-major representations transposes a the matrix of any length.)
The transpose operation shows up frequently, for example in the [least-squares linear regression estimator](https://en.wikipedia.org/wiki/Least_squares#Linear_least_squares).

We have just shown we can compute a matrix's transpose without reading or writing any of the elements!
For large matrices, this can save a lot of time.

To make use of this, we could define a slice "view" that additionally stores whether it's transposed:
```rust
struct Slice2DView<'a> {
  data: &'a [i32], // borrowed, so we can have multiple views for one allocation
  length0: usize,
  length1: usize,
  is_transposed: bool,
}
```
To compute the length of the view and a flattened index, we would have two cases depending on whether it's transposed:
```rust
impl Slice2DView<'_> {
  fn len(&self) -> (usize, usize) {
    if self.is_transposed {
      (self.length1, self.length0)
    }
    else {
      (self.length0, self.length1)
    }
  }

  fn index(&self, index0: usize, index1: usize) -> usize {
    if self.is_transposed {
      index0 + index1 * self.length0 // column-major order
    }
    else {
      index0 * self.length1 + index1 // row-major order
    }
  }
}
```

We could implement the rest of the methods as before, but it turns out there's an even better representation.
Start by noticing the similarity between the row-major and column-major index calculations.
If we add `* 1`, the symmetry is a bit more clear:
```rust
fn index(&self, index0: usize, index1: usize) -> usize {
  if self.is_transposed {
    index0 * 1 + index1 * self.length0
  }
  else {
    index0 * self.length1 + index1 * 1
  }
}
```
In each case, we multiply `index0` and `index1` by some constant and add them together.
Because the multiplier tells us how many elements we need to advance in the flattened slice in order to advance 1 element along that dimension, it's often called the dimension's "stride".
So instead of storing an `is_transposed` field, we can store the stride in each dimension.
(We will also need to update the lengths when we transpose the view.)
Here's what that looks like:
```rust
struct Slice2DView<'a> {
  data: &'a [i32], // borrowed, so we can have multiple views for one allocation
  length0: usize,
  length1: usize,
  /// How many elements to advance in `data`
  /// to advance 1 element along dimension 0
  stride0: usize,
  /// How many elements to advance in `data`
  /// to advance 1 element along dimension 1
  stride1: usize,
}
```
This significantly simplifies our `len()` and `index()` methods:
```rust
impl Slice2DView<'_> {
  fn len(&self) -> (usize, usize) {
    (self.length0, self.length1)
  }

  fn index(&self, index0: usize, index1: usize) -> usize {
    index0 * self.stride0 + index1 * self.stride1
  }
}
```
Computing the initial stride and the transposed stride isn't too complicated either:
```rust
impl Slice2D {
  /// Creates a (borrowed) view of the owned slice
  fn view(&self) -> Slice2DView<'_> {
    let Self { ref data, length0, length1 } = *self;
    // Slice is in row-major order, so initial stride is `(length1, 1)`
    Slice2DView { data, length0, length1, stride0: length1, stride1: 1 }
  }
}

impl Slice2DView<'_> {
  /// Creates a view that is the transpose of this view
  fn transpose(&self) -> Self {
    let Self { data, length0, length1, stride0, stride1 } = *self;
    // Lengths and strides are swapped to swap the meaning of the dimensions
    Self {
      data,
      length0: length1,
      length1: length0,
      stride0: stride1,
      stride1: stride0,
    }
  }
}
```
Adding a `get()` method to `Slice2DView`, we can now demonstrate transposing the slice:
```rust
impl Slice2DView<'_> {
  fn get(&self, index0: usize, index1: usize) -> i32 {
    self.data[self.index(index0, index1)]
  }
}

// This is the same as before, but now takes a Slice2DView, not a Slice2D
fn print_slice(slice: &Slice2DView) {
  let (length0, length1) = slice.len();
  for index0 in 0..length0 {
    for index1 in 0..length1 {
      print!("{} ", slice.get(index0, index1));
    }
    println!();
  }
}

fn main() {
  let mut slice = Slice2D::zero(2, 3);
  // Initialize the data since we haven't implemented `set()`
  for (element, i) in slice.data.iter_mut().zip(1..) {
    *element = i;
  }
  println!("Original slice:");
  let view = slice.view();
  print_slice(&view);
  println!("Transposed:");
  print_slice(&view.transpose());
}
```
This prints:
```
Original slice:
1 2 3
4 5 6
Transposed:
1 4
2 5
3 6
```

In the example above, we allocate a single 2-dimensional slice (represented as a `Slice2D`) and create multiple views of it (represented as `Slice2DView`s).
From an ownership perspective, `Slice2D` is analogous to a `Box<[i32]>` and `Slice2DView` serves the role of `&[i32]`.
(We can "dereference" a `Slice2D` into a `Slice2DView`, `Slice2DView` doesn't allow the elements to be mutated, `Slice2DView` can be `Copy`, etc.)
The missing analog to `&mut [i32]` would be something like `Slice2DViewMut`, allowing mutable access to the elements.

### Going unsafe

I promised that storing `data: &[i32]` in `Slice2DView` was unnecessary; we only need the pointer to the start of the data, not the length.
Currently, the program is safe because the length stored in `data` is used to check the bounds every time we index `data`.
However, we shouldn't rely on this bounds check to ensure the index we provide to `get()` is valid.
Since the elements are stored contiguously, we can provide an invalid index as long as `self.index(index0, index1)` stays within `data.len()`:
```rust
fn main() {
  // Make a 2-dimensional slice with length (3, 3)
  let mut slice = Slice2D::zero(3, 3);
  for (element, i) in slice.data.iter_mut().zip(1..) {
    *element = i;
  }
  let view = slice.view();
  // Access index (0, 4), which should be out of bounds
  println!("slice[0, 4] = {}", view.get(0, 4));
  // Oops, we end up getting (1, 1) instead!
}
```

So we really should be checking each index against that dimension's length.
Since we know `data` is large enough to store all our elements, we know we won't access out-of-bounds memory if `index0 < length0 && index1 < length1`.
Therefore, storing just the pointer to the start of `data` is enough.
(A `NonNull<i32>` would be even better, since it allows `Option<Slice2DView>` to have the same size as `Slice2DView`.)
```rust
struct Slice2DView {
  data: *const i32, // the address of the start of the data (index (0, 0))
  length0: usize,
  length1: usize,
  stride0: usize,
  stride1: usize,
}

impl Slice2D {
  fn view(&self) -> Slice2DView {
    let Self { ref data, length0, length1 } = *self;
    let data = data.as_ptr(); // only need the pointer to the first element
    Slice2DView { data, length0, length1, stride0: length1, stride1: 1 }
  }
}

impl Slice2DView {
  fn get(&self, index0: usize, index1: usize) -> i32 {
    // Both indices need to be within their bounds
    assert!(
      index0 < self.length0 && index1 < self.length1,
      "Invalid index ({}, {}) for length ({}, {})",
      index0, index1, self.length0, self.length1,
    );
    let index = self.index(index0, index1);
    // SAFETY: we checked that the index is in-bounds
    unsafe { *self.data.add(index) }
  }
}
```

Our transpose test program still works:
```
Original slice:
1 2 3
4 5 6
Transposed:
1 4
2 5
3 6
```
And our invalid access panics properly now:
```
thread 'main' panicked at 'Invalid index (0, 4) for length (3, 3)', src/main.rs:59:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## The power of the stride vector

Okay, storing the stride in each dimension is an alternate way to implement the transpose operation, \*shrug\*.

Well, it turns out that many other useful operations on N-dimensional slices can be done by manipulating the strides (collectively called the "stride vector").
This means these operations are cheap even on large slices (there is no need to copy the elements themselves).
And more importantly, since these operations all manipulate the stride vector, they can be chained together, all without copying.
Let's take a look at all the operations that are possible.

### Transposing

We've discussed the matrix transpose operation (swapping rows with columns) above.
Here was an example:
```
[
  [1, 2, 3],
  [4, 5, 6],
]
becomes
[
  [1, 4],
  [2, 5],
  [3, 6],
]
```
We implemented this by swapping the row stride with the column stride, and swapping the row length with the column length.

More generally, we can permute the dimensions any way we want.
For example, if a 3-dimensional slice was originally indexed by `(i, j, k)`, we could make a new view indexed by `(j, k, i)` or `(k, j, i)` instead.

### Slicing

Just like normal Rust slices (`&[T]`) can be sliced (`&some_slice[1..3]`), N-dimensional slices can be sliced along any dimension.
Here is an example, slicing `1..3` along dimension 1 (the columns):
```
[
  [1, 2, 3],
  [4, 5, 6],
  [7, 8, 9],
]
becomes
[
  [2, 3],
  [5, 6],
  [8, 9],
]
```

Note that slicing doesn't affect the dimensions (an N-dimensional slice would remain an N-dimensional slice after slicing it any way and the dimensions would still have the same meaning).
So the stride vector is unchanged.
To slice from `start` to `end` along dimension `d`, `start * stride[d]` is added to the starting pointer to skip the first `start` elements in that dimension, and `len[d]` becomes `end - start`.

### Skipping

Normal slices in Rust don't have a stride associated with them.
But what if they did?
A stride of 2, for example, would pick out only the even indices of the slice.
A stride of 3 would pick out every third element.
Since we are already storing the stride vector, we can easily apply an additional stride along any dimension.
Here is an example, applying stride 2 along dimension 1 (the columns):
```
[
  [1, 2, 3],
  [4, 5, 6],
  [7, 8, 9],
]
becomes
[
  [1, 3],
  [4, 6],
  [7, 9],
]
```

Applying a stride of `s` along dimension `d` doesn't change the starting pointer, but it *multiplies* `stride[d]` by `s` and *divides* `len[d]` by `s` (rounding up).

### Removing a dimension

So far, all the operations we've seen don't affect an N-dimensional slice's dimensions.
But we sometimes want to change the number of dimensions in an N-dimensional slice.
One example is picking out a single index along one of the dimensions.
For example, we could take a matrix (2 dimensions) and pick out only the elements with index 1 along dimension 0 (the rows), resulting in a 1-dimensional slice:
```
[
  [1, 2, 3],
  [4, 5, 6],
  [7, 8, 9],
]
becomes
[4, 5, 6]
```
Note that slicing `1..=1` along dimension 0 would give the same elements, but keep it a 2-dimensional slice:
```
[
  [4, 5, 6],
]
```

Picking out index 0 along dimension `d` simply removes dimension `d` from the stride and length vectors.
Picking out index `i` additionally requires adding `i * stride[d]` to the starting pointer, just as when slicing.
We end up with an `N - 1`-dimensional slice.

### Adding a dimension

Removing a dimension by picking out data at one index may seem reasonable, but how could we possibly *add* a new dimension without creating or copying any data?

Well, first consider the case where we want the new dimension to have length 1.
When going from a 1-dimensional slice to a 2-dimensional slice, this is called creating a row or column vector, depending on which dimension is added:
```
[1, 2, 3]
becomes (row vector)
[
  [1, 2, 3],
]
or (column vector)
[
  [1],
  [2],
  [3],
]
```
In this case, the underlying elements are exactly the same (by adding a dimension of length 1, we have multiplied the total number of elements by 1).
This would just require adding the length 1 to the length vector and any stride to the stride vector (it is only possible to access index 0 along this dimension, so the stride is irrelevant).

Let's try to generalize this: adding a dimension with length `l` would give an `N + 1`-dimensional slice with `l` copies of the original slice along the new dimension.
Can we represent an N-dimensional slice where every index along the new dimension refers to the same (original) data?
Actually, we can!
Since we want to advance along the new dimension without moving in the underlying slice, we need the dimension to have a *stride of 0*.
Using this trick, we could, for example, add a new dimension 0 with length 3:
```
[1, 2, 3]
becomes
[
  [1, 2, 3],
  [1, 2, 3],
  [1, 2, 3],
]
```

## Making it generic

So far, we've played with a 2-dimensional slice of `i32`s.
But the same concepts work for any N-dimensional slice of any (`Sized`) type `T`.
To extend the operations we've implemented to any possible slice, we would want to make it generic over both `N` and `T`.
With Rust's recent ["const generics"](https://blog.rust-lang.org/2021/02/26/const-generics-mvp-beta.html) features, this is actually doable with just a few unstable features.
N-dimensional indices, lengths, and strides can be represented as `[usize; N]`.

You can find the full implementation on [GitHub](https://github.com/calebsander/nd-slice) with some tests to illustrate how it can be used.
I won't bore you with all the details, but I want to give the gist of what it looks like.

### The interfaces

There are different N-dimensional slice types corresponding to the ownership of `Box`, `&`, and `&mut` (some helper types removed for clarity):
```rust
use std::marker::PhantomData;
use std::ptr::NonNull;

/// N-dimensional analog of Box<[T]>
pub struct NDBox<T, const N: usize> {
  data: NonNull<T>,
  len: [usize; N],
}

/// N-dimensional analog of &[T]
pub struct NDSlice<'a, T, const N: usize> {
  data: NonNull<T>,
  len: [usize; N],
  stride: [usize; N],
  phantom: PhantomData<&'a T>,
}

/// N-dimensional analog of &mut [T]
pub struct NDSliceMut<'a, T, const N: usize> {
  data: NonNull<T>,
  len: [usize; N],
  stride: [usize; N],
  phantom: PhantomData<&'a mut T>,
}
```

All N-dimensional slices start from an `NDBox`, which allocates the storage for the values as a `Box<[T]>` in row-major order.
An `NDBox` can be created most generally by providing the length in each dimension and an initializer function to create the value at each index.
There are also helpers to create a slice filled with a single value or the default value for `T`:
```rust
impl<T, const N: usize> NDBox<T, N> {
  pub fn new_with<F>(len: [usize; N], mut init: F) -> Self
    where F: FnMut([usize; N]) -> T
  {
    // ...
  }

  pub fn new_fill(len: [usize; N], value: T) -> Self where T: Clone {
    Self::new_with(len, |_| value.clone())
  }

  pub fn new_default(len: [usize; N]) -> Self where T: Default {
    Self::new_with(len, |_| T::default())
  }
}
```
And `From` implementations to write what look like `NDBox` literals (up to 2 dimensions):
```rust
/// 0-dimensional NDBox literal, e.g.:
/// NDBox::from(123)
impl<T> From<T> for NDBox<T, 0> {
  fn from(value: T) -> Self {
    // ...
  }
}

/// 1-dimensional NDBox literal, e.g.:
/// NDBox::from([1, 2, 3])
impl<T, const L0: usize> From<[T; L0]> for NDBox<T, 1> {
  fn from(value: [T; L0]) -> Self {
    // ...
  }
}

/// 2-dimensional NDBox literal, e.g.:
/// NDBox::from([
///   [1, 2, 3],
///   [4, 5, 6],
///   [7, 8, 9],
/// ])
impl<T, const L0: usize, const L1: usize> From<[[T; L1]; L0]> for NDBox<T, 2> {
  fn from(value: [[T; L1]; L0]) -> Self {
    // ...
  }
}
```

From an `NDBox`, you can create an `NDSlice` or `NDSliceMut` of all its elements:
```rust
impl<T, const N: usize> NDBox<T, N> {
  pub fn as_slice(&self) -> NDSlice<T, N> {
    let Self { data, len } = *self;
    NDSlice { data, len, stride: self.stride(), phantom: PhantomData }
  }

  pub fn as_mut(&mut self) -> NDSliceMut<T, N> {
    let Self { data, len } = *self;
    NDSliceMut { data, len, stride: self.stride(), phantom: PhantomData }
  }
}
```
An `NDSliceMut` can also be borrowed as an `NDSlice`:
```rust
impl<'a, T, const N: usize> NDSliceMut<'a, T, N> {
  pub fn as_slice(&self) -> NDSlice<'a, T, N> {
    let Self { data, len, stride, .. } = *self;
    NDSlice { data, len, stride, phantom: PhantomData }
  }
}
```
Just like `&T`, `NDSlice` is `Copy`, so we can have multiple (read-only) slices of the same data:
```rust
impl<T, const N: usize> Clone for NDSlice<'_, T, N> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<T, const N: usize> Copy for NDSlice<'_, T, N> {}
```

As with `&[T]` and `&mut [T]`, elements can be accessed using:
- The `Index`/`IndexMut` operators, which panic if the index is out of bounds
- The `get()`/`get_mut()` methods, which return `None` on out-of-bounds
- The `unsafe` `get_unchecked()`/`get_unchecked_mut()` methods, which assume the index is in bounds

Both `NDSlice` and `NDSliceMut` can be manipulated using the stride and length vector operations discussed above:
```rust
/// Since `True` is only implemented for `Is<true>`,
/// the constraint `Is<X>: True` requires `X` to be true.
pub enum Is<const B: bool> {}
pub trait True {}
impl True for Is<true> {}

/// A range along a dimension, along with a number of indices to skip in between.
/// (Builder methods omitted for brevity.)
pub struct Bounds {
  start: Option<usize>,
  end: Option<usize>,
  step: usize,
}

impl<'a, T, const N: usize> NDSlice<'a, T, N> {
  /// Picks out the elements at a given index along dimension `D`.
  /// The dimension is required to be a constant so it can be checked at compile time.
  pub fn extract<const D: usize>(self, dimension_index: usize)
    -> NDSlice<'a, T, {N - 1}>
    where Is<{D < N}>: True
  {
    // ...
  }

  /// Adds a new dimension at index `D` with the given length.
  /// Picking out any index along the new dimension will give the original slice.
  /// The dimension is required to be a constant so it can be checked at compile time.
  pub fn add_dimension<const D: usize>(self, dimension_len: usize)
    -> NDSlice<'a, T, {N + 1}>
    where Is<{D <= N}>: True
  {
    // ...
  }

  /// Restricts the array to a slice along each dimension.
  /// Also allows applying an additional stride with Bounds::step().
  /// To leave a dimension unsliced, use Bounds::all() as its bounds.
  pub fn slice(self, bounds: [Bounds; N]) -> Self {
    // ...
  }

  /// Reverses the dimensions, so what was at index [a, ..., z] becomes index [z, ..., a].
  /// For a 2-dimensional slice, this is the matrix transpose operation.
  pub fn transpose(self) -> Self {
    // ...
  }
}
```

### The implementation

#### Allocation size, default stride vector, and indexing

Generalizing the 2-dimensional case, the total number of elements is computed by multiplying the lengths in each dimension.
```rust
fn size(len: [usize; N]) -> usize {
  len.iter().product()
}
```

Since `NDBox` elements are stored in row-major order, the stride is computed as the cumulative products of the final dimension lengths:
```rust
fn default_stride(len: [usize; N]) -> [usize; N] {
  // Row-major order: indices are ordered by dimension 0, then 1, ..., N - 1.
  // So dimension N - 1 has stride 1, dimension N - 2 has stride len[N - 1], etc.
  let mut stride = [0; N];
  let mut next_stride = 1;
  for (dimension_stride, dimension_len) in iter::zip(&mut stride, len).rev() {
    *dimension_stride = next_stride;
    next_stride *= dimension_len;
  }
  stride
}
```

The location of an element is computed by multiplying each dimension index against the corresponding stride and offsetting it from the first element:
```rust
use std::iter;

impl<'a, T, const N: usize> NDSlice<'a, T, N> {
  /// SAFETY: each dimension index must be at most the corresponding dimension length
  /// (so the resulting pointer does not go past the end of the underlying allocation)
  unsafe fn location(self, index: [usize; N]) -> NonNull<T> {
    let offset = iter::zip(index, self.stride)
      .map(|(dimension_index, dimension_stride)| {
        dimension_index * dimension_stride
      })
      .sum();
    NonNull::new_unchecked(self.data.as_ptr().add(offset))
  }
}
```

#### `extract()`

`extract()` is implemented by offsetting to the specified index along the dimension and removing that dimension from the length and stride vectors:
```rust
fn remove<const N: usize, const I: usize>(input: [usize; N]) -> [usize; N - 1]
  where Is<{I < N}>: True
{
  let mut result = [0; N - 1];
  result[..I].copy_from_slice(&input[..I]);
  result[I..].copy_from_slice(&input[I + 1..]);
  result
}

impl<'a, T, const N: usize> NDSlice<'a, T, N> {
  pub fn extract<const D: usize>(self, dimension_index: usize)
    -> NDSlice<'a, T, {N - 1}>
    where Is<{D < N}>: True
  {
    let Self { len, stride, .. } = self;
    let mut index = [0; N];
    index[D] = dimension_index;
    let data = unsafe { self.location(index) };
    let len = remove::<N, D>(len);
    let stride = remove::<N, D>(stride);
    NDSlice { data, len, stride, phantom: PhantomData }
  }
}
```

#### `add_dimension()`

`add_dimension()` is implemented by simply inserting the new dimension length into the length vector and stride 0 into the stride vector at the given dimension:
```rust
fn insert<const N: usize, const I: usize>(input: [usize; N], value: usize)
  -> [usize; N + 1]
  where Is<{I <= N}>: True
{
  let mut result = [value; N + 1];
  result[..I].copy_from_slice(&input[..I]);
  result[I + 1..].copy_from_slice(&input[I..]);
  result
}

impl<'a, T, const N: usize> NDSlice<'a, T, N> {
  pub fn add_dimension<const D: usize>(self, dimension_len: usize)
    -> NDSlice<'a, T, {N + 1}>
    where Is<{D <= N}>: True
  {
    let Self { data, len, stride, .. } = self;
    let len = insert::<N, D>(len, dimension_len);
    let stride = insert::<N, D>(stride, 0);
    NDSlice { data, len, stride, phantom: PhantomData }
  }
}
```

#### `slice()`

`slice()` allows every dimension to be sliced simultaneously so the syntax can be similar to indexing:
```rust
// Assuming nd_slice is a 2-dimensional slice (matrix),
// slice only the even rows, and columns 1 to 3
nd_slice.slice([Bounds::all().step(2), Bounds::all().from(1).to(3)])
```
The data pointer is offset to the starting index in each dimension.
The new length in each dimension is the difference between the starting and ending indices, divided by the step size (rounding up).
And the stride in each dimension is multiplied by the step size.
```rust
impl<'a, T, const N: usize> NDSlice<'a, T, N> {
  pub fn slice(self, bounds: [Bounds; N]) -> Self {
    let Self { len, stride, .. } = self;
    let dimensions = bounds.zip(len).zip(stride)
      .map(|((dimension_bounds, dimension_len), dimension_stride)| {
        let dimension_start = dimension_bounds.start.unwrap_or(0);
        let dimension_end = dimension_bounds.end.unwrap_or(dimension_len);
        let dimension_len = (dimension_start..dimension_end)
          .step_by(dimension_bounds.step)
          .len();
        let dimension_stride = dimension_stride * dimension_bounds.step;
        (dimension_start, dimension_len, dimension_stride)
      });
    let index = dimensions.map(|(dimension_start, _, _)| dimension_start);
    let data = unsafe { self.location(index) };
    let len = dimensions.map(|(_, dimension_len, _)| dimension_len);
    let stride = dimensions.map(|(_, _, dimension_stride)| dimension_stride);
    Self { data, len, stride, phantom: PhantomData }
  }
}
```

#### `transpose()`

`transpose()` simply reverses the length and stride vectors, which reverses the meaning of indices in the index vector:
```rust
impl<'a, T, const N: usize> NDSlice<'a, T, N> {
  pub fn transpose(mut self) -> Self {
    self.len.0.reverse();
    self.stride.0.reverse();
    self
  }
}
```

## An example

As a practical example of what we can now do, let's consider a dataset with the high temperatures over 10 days in 3 cities:
```rust
let temperatures_fahrenheit = NDBox::<f32, 2>::from([
  // NYC, LAX, CHI
  [72.0, 80.0, 79.0], // 2022-06-01
  [79.0, 79.0, 79.0], // 2022-06-02
  [76.0, 73.0, 83.0], // 2022-06-03
  [80.0, 70.0, 72.0], // 2022-06-04
  [77.0, 75.0, 81.0], // 2022-06-05
  [80.0, 77.0, 76.0], // 2022-06-06
  [78.0, 76.0, 71.0], // 2022-06-07
  [82.0, 75.0, 72.0], // 2022-06-08
  [81.0, 80.0, 80.0], // 2022-06-09
  [77.0, 81.0, 82.0], // 2022-06-10
]);
let [days, cities] = temperatures_fahrenheit.len();
```

First, let's convert the temperatures to Celsius.
Since `temp_C = (temp_F - 32) / 1.8`, we create two slices of the same length as `temperatures_fahrenheit`, one filled with 32 and one with 1.8.
Using `add_dimension()`, we only need to allocate 1 element for each slice:
```rust
let const_32 = NDBox::from(32.0);
let const_32 = const_32.as_slice()
  .add_dimension::<0>(days)
  .add_dimension::<1>(cities);
let const_1_8 = NDBox::from(1.8);
let const_1_8 = const_1_8.as_slice()
  .add_dimension::<0>(days)
  .add_dimension::<1>(cities);
dbg!(const_32);
```
Printing out `const_32` shows it has the value `32.0` for each day and city:
```
[src/main.rs:28] const_32 = [
    [32.0, 32.0, 32.0],
    [32.0, 32.0, 32.0],
    [32.0, 32.0, 32.0],
    [32.0, 32.0, 32.0],
    [32.0, 32.0, 32.0],
    [32.0, 32.0, 32.0],
    [32.0, 32.0, 32.0],
    [32.0, 32.0, 32.0],
    [32.0, 32.0, 32.0],
    [32.0, 32.0, 32.0],
]
```
Now we can use the binary operators implemented on slices to perform the Celsius conversion:
```rust
let temperatures_celsius =
  (temperatures_fahrenheit.as_slice() - const_32) / const_1_8;
let temperatures_celsius = temperatures_celsius.as_slice();
dbg!(temperatures_celsius);
```
The Celsius values are:
```
[src/main.rs:35] temperatures_celsius = [
    [22.222223, 26.666668, 26.111113],
    [26.111113, 26.111113, 26.111113],
    [24.444445, 22.777779, 28.333334],
    [26.666668, 21.111113, 22.222223],
    [25.0, 23.88889, 27.222223],
    [26.666668, 25.0, 24.444445],
    [25.555555, 24.444445, 21.666668],
    [27.777779, 23.88889, 22.222223],
    [27.222223, 26.666668, 26.666668,],
    [25.0, 27.222223, 27.777779],
]
```
We can then compute the average temperatures for each city by extracting that city's temperatures and averaging them:
```rust
let average_temperatures = NDBox::new_with([cities], |[city]| {
  let city_temperatures = temperatures_celsius.extract::<1>(city);
  city_temperatures.into_iter().sum::<f32>() / days as f32
});
let average_temperatures = average_temperatures.as_slice();
dbg!(average_temperatures);
```
The result is a 1-dimensional slice:
```
[src/main.rs:42] average_temperatures = [
    25.666668,
    24.777779,
    25.27778,
]
```

## Missing features

If you want to tinker with the codebase, here are some features it should probably support but doesn't at the moment:
- Slicing in reverse (without copying!): for example, turn the view `[1, 2, 3]` into `[3, 2, 1]`.
  This could be accomplished by an additional `bool` indicating whether an `NDSlice` is reversed in each dimension.
  Could you do it with just the stride vector instead? :)
- Permuting dimensions arbitrarily: currently, `transpose()` just reverses the dimensions, but any permutation should be allowed (see ["Transposing"](#transposing))

These are marked with `TODO` in the code.
