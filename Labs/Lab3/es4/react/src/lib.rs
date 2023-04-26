// TODO: Make code not look like the example code. It is correct but I don't like it.

use core::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::fmt::Debug;

/// `InputCellId` is a unique identifier for an input cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InputCellId(usize);

static CURRENT_CELL_ID: AtomicUsize = AtomicUsize::new(0);
static CURRENT_CALLBACK_ID: AtomicUsize = AtomicUsize::new(0);
static CURRENT_COMPUTE_ID: AtomicUsize = AtomicUsize::new(0);

impl InputCellId {
    fn new() -> Self {
        let id = CURRENT_CELL_ID.load(Ordering::Relaxed);
        CURRENT_CELL_ID.store(id + 1, Ordering::Relaxed);
        InputCellId(id)
    }
}

/// `ComputeCellId` is a unique identifier for a compute cell.
/// Values of type `InputCellId` and `ComputeCellId` should not be mutually assignable,
/// demonstrated by the following tests:
///
/// ```compile_fail
/// let mut r = react::Reactor::new();
/// let input: react::ComputeCellId = r.create_input(111);
/// ```
///
/// ```compile_fail
/// let mut r = react::Reactor::new();
/// let input = r.create_input(111);
/// let compute: react::InputCellId = r.create_compute(&[react::CellId::Input(input)], |_| 222).unwrap();
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ComputeCellId(usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CallbackId(usize);

impl ComputeCellId {
    fn new() -> Self {
        let id = CURRENT_COMPUTE_ID.load(Ordering::Relaxed);
        CURRENT_COMPUTE_ID.store(id + 1, Ordering::Relaxed);
        ComputeCellId(id)
    }
}

impl CallbackId {
    fn new() -> Self {
        let id = CURRENT_CALLBACK_ID.load(Ordering::Relaxed);
        CURRENT_CALLBACK_ID.store(id + 1, Ordering::Relaxed);
        CallbackId(id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CellId {
    Input(InputCellId),
    Compute(ComputeCellId),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RemoveCallbackError {
    NonexistentCell,
    NonexistentCallback,
}

pub struct ValueWithMemory<T: Copy + Debug + PartialEq> {
    value: T,
    last_value: T,
}

impl<T: Copy + PartialEq + Debug> ValueWithMemory<T> {
    fn new(value: T) -> Self {
        ValueWithMemory {
            value,
            last_value: value,
        }
    }

    fn get(&self) -> T {
        self.value
    }

    fn has_changed(&self) -> bool {
        self.value != self.last_value
    }

    fn set(&mut self, value: T) {
        self.value = value;
    }

    fn set_last(&mut self, value: T) {
        self.last_value = value;
    }
}

pub struct Cell<T: Copy + Debug + PartialEq> {
    value: ValueWithMemory<T>,
    dependents: Vec<ComputeCellId>,
}

type BoxedFn<'a, T> = Box<dyn 'a + Fn(&[T]) -> T>;

pub struct ComputeCell<'a, T: Copy + Debug + PartialEq> {
    cell: Cell<T>,
    deps: Vec<CellId>,
    fun: BoxedFn<'a, T>,
    callbacks: HashMap<CallbackId, Box<dyn 'a + FnMut(T)>>,
}

impl<T: Copy + PartialEq + Debug> Cell<T> {
    fn new(initial: T) -> Self {
        Cell {
            value: ValueWithMemory::new(initial),
            dependents: Vec::new(),
        }
    }
    fn update(&mut self, new_value: T) {
        self.value.set(new_value)
    }
}

impl<'a, T: Copy + PartialEq + Debug> ComputeCell<'a, T> {
    fn new<F: 'a + Fn(&[T]) -> T>(initial: T, deps: Vec<CellId>, f: F) -> Self {
        ComputeCell {
            cell: Cell::new(initial),
            deps,
            fun: Box::new(f),
            callbacks: HashMap::new(),
        }
    }
}

pub struct Reactor<'a, T: Copy + Debug + PartialEq> {
    inputs: HashMap<InputCellId, Cell<T>>,
    cells: HashMap<ComputeCellId, ComputeCell<'a, T>>,
}

// You are guaranteed that Reactor will only be tested against types that are Copy + PartialEq.
impl<'a, T: Copy + PartialEq + Debug> Reactor<'a, T> {
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            cells: HashMap::new(),
        }
    }

    // Creates an input cell with the specified initial value, returning its ID.
    pub fn create_input(&mut self, initial: T) -> InputCellId {
        let id = InputCellId::new();
        self.inputs.insert(id, Cell::new(initial));
        id
    }

    // Creates a compute cell with the specified dependencies and compute function.
    // The compute function is expected to take in its arguments in the same order as specified in
    // `dependencies`.
    // You do not need to reject compute functions that expect more arguments than there are
    // dependencies (how would you check for this, anyway?).
    //
    // If any dependency doesn't exist, returns an Err with that nonexistent dependency.
    // (If multiple dependencies do not exist, exactly which one is returned is not defined and
    // will not be tested)
    //
    // Notice that there is no way to *remove* a cell.
    // This means that you may assume, without checking, that if the dependencies exist at creation
    // time they will continue to exist as long as the Reactor exists.
    pub fn create_compute<F: 'a + Fn(&[T]) -> T>(
        &mut self,
        dependencies: &[CellId],
        compute_func: F,
    ) -> Result<ComputeCellId, CellId> {
        let compute_id = ComputeCellId::new();
        for dep in dependencies.iter() {
            match dep {
                CellId::Input(id) => match self.inputs.get_mut(id) {
                    Some(cell) => cell.dependents.push(compute_id),
                    None => return Err(*dep),
                },
                CellId::Compute(id) => match self.cells.get_mut(id) {
                    Some(cell) => cell.cell.dependents.push(compute_id),
                    None => return Err(*dep),
                },
            }
        }
        let inputs = dependencies
            .iter()
            .map(|&id| self.value(id).unwrap())
            .collect::<Vec<T>>();
        let initial = compute_func(&inputs);
        self.cells.insert(
            compute_id,
            ComputeCell::new(initial, dependencies.to_vec(), compute_func),
        );
        Ok(compute_id)
    }

    // Retrieves the current value of the cell, or None if the cell does not exist.
    //
    // You may wonder whether it is possible to implement `get(&self, id: CellId) -> Option<&Cell>`
    // and have a `value(&self)` method on `Cell`.
    //
    // It turns out this introduces a significant amount of extra complexity to this exercise.
    // We chose not to cover this here, since this exercise is probably enough work as-is.
    pub fn value(&self, id: CellId) -> Option<T> {
        match id {
            CellId::Input(id) => self.inputs.get(&id).map(|c| c.value.get()),
            CellId::Compute(id) => self.cells.get(&id).map(|c| c.cell.value.get()),
        }
    }

    fn update_dependent(&mut self, id: ComputeCellId) {
        let (new_value, deps) = {
            let (dependencies, f, dependents) = match self.cells.get(&id) {
                Some(c) => (&c.deps, &c.fun, c.cell.dependents.clone()),
                None => return,
            };
            let inputs = dependencies
                .iter()
                .map_while(|&id| self.value(id))
                .collect::<Vec<T>>();
            (f(&inputs), dependents)
        };
        if let Some(comp) = self.cells.get_mut(&id) {
            if comp.cell.value.get() == new_value {
                return;
            }
            comp.cell.update(new_value);
        }
        for dep in deps {
            self.update_dependent(dep);
        }
    }

    fn fire_callbacks(&mut self, id: ComputeCellId) {
        let dependents = match self.cells.get_mut(&id) {
            Some(c) => {
                if !c.cell.value.has_changed() {
                    return;
                }
                for cb in c.callbacks.values_mut() {
                    cb(c.cell.value.get());
                }
                c.cell.value.set_last(c.cell.value.get());
                c.cell.dependents.clone()
            }
            None => return,
        };
        for dep in dependents {
            self.fire_callbacks(dep);
        }
    }

    // Sets the value of the specified input cell.
    //
    // Returns false if the cell does not exist.
    //
    // Similarly, you may wonder about `get_mut(&mut self, id: CellId) -> Option<&mut Cell>`, with
    // a `set_value(&mut self, new_value: T)` method on `Cell`.
    //
    // As before, that turned out to add too much extra complexity.
    pub fn set_value(&mut self, id: InputCellId, new_value: T) -> bool {
        self.inputs
            .get_mut(&id)
            .map(|c| {
                c.update(new_value);
                c.dependents.clone()
            })
            .map(|deps| {
                for &dep in deps.iter() {
                    self.update_dependent(dep);
                }

                for dep in deps {
                    self.fire_callbacks(dep);
                }
            })
            .is_some()
    }

    // Adds a callback to the specified compute cell.
    //
    // Returns the ID of the just-added callback, or None if the cell doesn't exist.
    //
    // Callbacks on input cells will not be tested.
    //
    // The semantics of callbacks (as will be tested):
    // For a single set_value call, each compute cell's callbacks should each be called:
    // * Zero times if the compute cell's value did not change as a result of the set_value call.
    // * Exactly once if the compute cell's value changed as a result of the set_value call.
    //   The value passed to the callback should be the final value of the compute cell after the
    //   set_value call.
    pub fn add_callback<F: 'a + FnMut(T)>(
        &mut self,
        id: ComputeCellId,
        callback: F,
    ) -> Option<CallbackId> {
        self.cells.get_mut(&id).map(|c| {
            let callback_id = CallbackId::new();
            c.callbacks.insert(callback_id, Box::new(callback));
            callback_id
        })
    }

    // Removes the specified callback, using an ID returned from add_callback.
    //
    // Returns an Err if either the cell or callback does not exist.
    //
    // A removed callback should no longer be called.
    pub fn remove_callback(
        &mut self,
        cell: ComputeCellId,
        callback: CallbackId,
    ) -> Result<(), RemoveCallbackError> {
        match self.cells.get_mut(&cell) {
            Some(cell) => match cell.callbacks.remove(&callback) {
                Some(_) => Ok(()),
                None => Err(RemoveCallbackError::NonexistentCallback),
            },
            None => Err(RemoveCallbackError::NonexistentCell),
        }
    }
}

impl<'a, T: Copy + PartialEq + Debug> Default for Reactor<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}