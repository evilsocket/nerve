import os
import queue
import threading
import time
import typing as t


class ThreadPool:
    def __init__(self, num_workers: int | None = None):
        self.tasks = queue.Queue()  # type: ignore
        self.results: dict[int, t.Any] = {}
        self.workers: list[threading.Thread] = []
        self.shutdown_flag = threading.Event()
        self.num_workers = num_workers or os.cpu_count() or 4
        self.task_counter = 0
        self.task_counter_lock = threading.Lock()

        # Create and start worker threads
        for _ in range(self.num_workers):
            worker = threading.Thread(target=self._worker_loop)
            worker.daemon = True
            worker.start()
            self.workers.append(worker)

    def _worker_loop(self) -> None:
        while not self.shutdown_flag.is_set():
            try:
                # Get task with timeout to periodically check shutdown flag
                task_id, func, args, kwargs = self.tasks.get(timeout=0.5)
                try:
                    result = func(*args, **kwargs)
                    self.results[task_id] = result
                except Exception as e:
                    self.results[task_id] = e
                finally:
                    self.tasks.task_done()
            except queue.Empty:
                continue

    def submit(self, func: t.Callable, *args, **kwargs) -> int:  # type: ignore
        """Submit a function to be executed by thread pool"""
        with self.task_counter_lock:
            self.task_counter += 1
            task_id = self.task_counter

        self.tasks.put((task_id, func, args, kwargs))
        return task_id

    def wait_for_task(self, task_id: int, timeout: float | None = None) -> t.Any:
        """Wait for a specific task to complete and return its result"""
        start_time = time.time()
        while task_id not in self.results:
            if timeout is not None and time.time() - start_time > timeout:
                raise TimeoutError(f"Task {task_id} did not complete within timeout")
            time.sleep(0.01)

        result = self.results[task_id]
        if isinstance(result, Exception):
            raise result
        return result

    def wait_all(self) -> None:
        """Wait for all tasks to complete"""
        self.tasks.join()

    def shutdown(self) -> None:
        """Signal workers to exit and wait for them to terminate"""
        self.shutdown_flag.set()
        for worker in self.workers:
            worker.join()
