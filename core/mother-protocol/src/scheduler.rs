//! FastLink Scheduler Module
//!
//! Task scheduling and priority management

use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

pub struct ScheduledTask {
    pub id: u64,
    pub priority: i32,
    pub deadline: Instant,
    #[allow(clippy::box_collection)]
    pub task: Box<dyn FnOnce() + Send + 'static>,
}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ScheduledTask {}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => self.deadline.cmp(&other.deadline),
            other => other,
        }
    }
}

pub struct TaskScheduler {
    tasks: BinaryHeap<ScheduledTask>,
    next_id: u64,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            tasks: BinaryHeap::new(),
            next_id: 0,
        }
    }
    
    pub fn schedule<F>(&mut self, priority: i32, delay: Duration, task: F) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        
        let scheduled_task = ScheduledTask {
            id,
            priority,
            deadline: Instant::now() + delay,
            task: Box::new(task),
        };
        
        self.tasks.push(scheduled_task);
        id
    }
    
    pub fn schedule_at(&mut self, priority: i32, deadline: Instant, task: impl FnOnce() + Send + 'static) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        
        let scheduled_task = ScheduledTask {
            id,
            priority,
            deadline,
            task: Box::new(task),
        };
        
        self.tasks.push(scheduled_task);
        id
    }
    
    pub fn poll(&mut self) -> Option<Box<dyn FnOnce() + Send + 'static>> {
        if let Some(task) = self.tasks.peek() {
            if task.deadline <= Instant::now() {
                let task = self.tasks.pop().unwrap();
                return Some(task.task);
            }
        }
        None
    }
    
    pub fn next_deadline(&self) -> Option<Duration> {
        self.tasks.peek().map(|task| {
            if task.deadline > Instant::now() {
                task.deadline - Instant::now()
            } else {
                Duration::from_secs(0)
            }
        })
    }
    
    pub fn cancel(&mut self, id: u64) -> bool {
        let original_len = self.tasks.len();
        self.tasks.retain(|task| task.id != id);
        self.tasks.len() < original_len
    }
    
    pub fn clear(&mut self) {
        self.tasks.clear();
    }
    
    pub fn len(&self) -> usize {
        self.tasks.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_scheduler_basic() {
        let mut scheduler = TaskScheduler::new();
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);
        
        scheduler.schedule(0, Duration::from_millis(10), move || {
            *counter_clone.lock().unwrap() += 1;
        });
        
        assert_eq!(scheduler.len(), 1);
        assert_eq!(*counter.lock().unwrap(), 0);
    }

    #[test]
    fn test_scheduler_priority() {
        let mut scheduler = TaskScheduler::new();
        let order = Arc::new(Mutex::new(Vec::new()));
        let order1 = Arc::clone(&order);
        let order2 = Arc::clone(&order);
        
        scheduler.schedule(1, Duration::from_secs(0), move || {
            order1.lock().unwrap().push(1);
        });
        
        scheduler.schedule(0, Duration::from_secs(0), move || {
            order2.lock().unwrap().push(0);
        });
        
        // Execute highest priority task
        if let Some(task) = scheduler.poll() {
            task();
        }
        
        let result = order.lock().unwrap().clone();
        assert_eq!(result, vec![0]);
    }
    
    #[test]
    fn test_scheduler_multiple_tasks() {
        let mut scheduler = TaskScheduler::new();
        let order = Arc::new(Mutex::new(Vec::new()));
        
        // Schedule tasks with different priorities
        for i in 0..5 {
            let order_clone = Arc::clone(&order);
            scheduler.schedule(i, Duration::from_secs(0), move || {
                order_clone.lock().unwrap().push(i);
            });
        }
        
        // Execute all tasks in priority order
        while let Some(task) = scheduler.poll() {
            task();
        }
        
        let result = order.lock().unwrap().clone();
        // Should execute in priority order: 0, 1, 2, 3, 4
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }
}
