from __future__ import annotations
from typing import *

from PyQt5.QtWidgets import QProgressBar, QProgressDialog, QApplication
from PyQt5.QtCore import pyqtSignal, pyqtSlot, QRunnable, QObject, QThreadPool, QThread


T = TypeVar("T") # User data
R = TypeVar("R") # Return value

def run_in_background(
	task: Callable[[pyqtSignal[int], pyqtSignal[int], pyqtSignal[T]], R],
	progress_bar: Union[QProgressBar, QProgressDialog],
	user_data_callback: Callable[[T], None],
	finished_callback: Callable[[R], None],
) -> Tuple[QThread, QObject]:
	"""
	Utility function to run stuff in background with a progress bar.
	Expects a `task` taking three pyqt signals as arguments, where the first one is a signal to emit
	the maximum progress value on, the second one is the actual progress value signal, and the third
	one is for arbitrary user data (when emitted, user_data_callback will be called with the sent
	value in the UI thread).
	`progress_bar` is a QProgressBar or QProgressDialog object for feedback.
	`finished_callback` is provided with only one argument, being the return value of `task`.
	This function returns a tuple of Qt objects that need to be saved to avoid deletion by GC.
	"""
	
	class WorkerObject(QObject):
		maximum = pyqtSignal(int) # for progress bar
		progress = pyqtSignal(int) # for progress bar
		user_data = pyqtSignal(object) # for user data
		finished = pyqtSignal(object) # for return value
		
		def __init__(self, task):
			super().__init__()
			self.task = task
		
		def run(self):
			result = (self.task)(self.maximum, self.progress, self.user_data)
			self.finished.emit(result)
	
	thread = QThread()
	obj = WorkerObject(task)
	
	# Connect signals
	obj.maximum.connect(lambda maximum: progress_bar.setMaximum(maximum))
	obj.progress.connect(lambda value: progress_bar.setValue(value))
	obj.user_data.connect(user_data_callback)
	def finished(value):
		progress_bar.setValue(progress_bar.maximum())
		thread.quit()
		(finished_callback)(value)
	obj.finished.connect(finished)
	
	# Do thread moving and start
	obj.moveToThread(thread)
	thread.started.connect(obj.run) # type: ignore
	thread.start()
	
	# Caller: Please save these values from GC
	return (thread, obj)

# `task` is a callback with three parameters:
# 1. A signal to emit the maximum progress value
# 2. A signal to emit the current progress value
# 3. A signal to emit the current progress text
def blocking_loading_bar(
	task: Callable[[pyqtSignal, pyqtSignal, pyqtSignal], R],
	window_title: str,
) -> R:
	progress_dialog = QProgressDialog()
	progress_dialog.setWindowTitle(window_title)
	progress_dialog.show()

	return_value = None
	def finished_callback(value: R) -> None:
		nonlocal return_value
		return_value = value
	
	def label_text_callback(new_label_text: str) -> None:
		progress_dialog.setLabelText(new_label_text)

	(thread, obj) = run_in_background(task, progress_dialog, label_text_callback, finished_callback)

	qapp = QApplication.instance()
	while not return_value:
		qapp.processEvents()

	del thread, obj

	progress_dialog.close()
	return return_value