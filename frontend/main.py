from __future__ import annotations
from typing import *

import time, logging
from dataclasses import dataclass

from PyQt5.Qt import QIcon
from PyQt5.QtWidgets import QPushButton, QApplication, QMainWindow, QTabWidget, QLabel, QMessageBox
from PyQt5.QtWidgets import QWidget, QGridLayout, QVBoxLayout, QRadioButton, QFrame, QDialog
from PyQt5.QtWidgets import QDialogButtonBox, QLineEdit, QStyle

import backend
import texts
from path_input import request_etterna_profile_paths
from loading_bar import blocking_loading_bar
from tab_widget_unlockable import TabWidgetUnlockable


def confirm_operation(title: str, message: str) -> bool:
	msgbox = QMessageBox(QMessageBox.Question, title, message, QMessageBox.Ok | QMessageBox.Cancel)
	clicked_button = msgbox.exec()

	return clicked_button == QMessageBox.Ok

def horizontal_separator() -> QWidget:
	line = QFrame()
	line.setFrameShape(QFrame.HLine)
	line.setFrameShadow(QFrame.Sunken)
	return line

class MainTabWidget(TabWidgetUnlockable):
	_replays_analysis: Optional[backend.ReplaysAnalysis]
	_charts_analysis: Optional[backend.ChartsAnalysis]

	def __init__(self, xml_stats: backend.XmlStats):
		super().__init__()

		self._replays_analysis = None
		self._charts_analysis = None

		self.setTabShape(QTabWidget.Triangular)
		
		self.addTab(XmlStatsTab(xml_stats), "XML data")
		self.addUnlockableTab(self._unlock_replay_stats, "Replay data")
		self.addUnlockableTab(self._unlock_chart_stats, "Chart data")
	
	def _unlock_replay_stats(self) -> Optional[QWidget]:
		if confirm_operation("Load replays",
			"In order to display these stats, the program needs to read and analyse "
			+ "your entire replay data. This may take a while"
		):
			self._replays_analysis = blocking_loading_bar(backend.load_replays_analysis, "Analysing replays...")
			return ReplaysStatsTab(self._replays_analysis)
		else:
			return None

	def _unlock_chart_stats(self) -> Optional[QWidget]:
		if not self._replays_analysis:
			QMessageBox.information(
				None,
				"Replays required",
				"First, replays need to be analyzed (press the replays data tab)"
			)
			return False

		if confirm_operation("Load charts",
			"In order to display these stats, the program needs to read and analyse "
			+ "your entire chart collection. This may take quite a while"
		):
			operation = lambda *args: backend.load_charts_analysis(self._replays_analysis, *args)
			self._charts_analysis = blocking_loading_bar(operation, "Loading charts...")
			return ChartsStatsTab(self._charts_analysis)
		else:
			return None

class XmlStatsTab(QWidget):
	def __init__(self, stats: backend.XmlStats):
		super().__init__()

		layout = QVBoxLayout()
		self.setLayout(layout)

		layout.addWidget(QLabel(str(stats.foo)))

class ReplaysStatsTab(QLabel):
	def __init__(self, replays_analysis: backend.ReplaysAnalysis):
		super().__init__("Replay stuff: " + repr(replays_analysis))

class ChartsStatsTab(QLabel):
	def __init__(self, charts_analysis: backend.ChartsAnalysis):
		super().__init__("Charts stuff: " + repr(charts_analysis))

# `options` list must have at least one entry
def choose_profile(options: List[backend.EtternaInstallation]) -> backend.EtternaProfilePaths:
	dialog = QDialog()
	layout = QVBoxLayout()
	dialog.setLayout(layout)

	if len(options) == 1:
		layout.addWidget(QLabel("An Etterna installation was automatically detected:"))
	else:
		layout.addWidget(QLabel("Multiple Etterna installations/profiles were detected:"))

	largest_xml_option_index = max(enumerate(options), key=lambda a: a[1].xml_size_mb)[0]
	
	radio_buttons = [
		QRadioButton(f"{option.base}\n{option.xml_size_mb:.1f}MB XML, {option.num_replays} replays")
		for option in options
	]
	radio_buttons[largest_xml_option_index].setChecked(True)
	for radio_button in radio_buttons: layout.addWidget(radio_button)
	
	button_box = QDialogButtonBox()
	layout.addWidget(button_box)

	return_value = None
	def submit_return_value(value: backend.EtternaProfilePaths) -> None:
		nonlocal return_value
		return_value = value
		dialog.accept()

	def confirm_choice() -> backend.EtternaProfilePaths:
		for i, radio_button in enumerate(radio_buttons):
			if radio_button.isChecked():
				return options[i].paths
		
		logging.warning("User somehow managed to confirm without having selected any of the checkboxes. Falling back to largest found xml")
		return options[largest_xml_option_index].paths
	button_box.addButton("Confirm", QDialogButtonBox.AcceptRole)\
		.clicked.connect(lambda: submit_return_value(confirm_choice()))
	button_box.addButton("Choose custom", QDialogButtonBox.AcceptRole)\
		.clicked.connect(lambda: submit_return_value(request_etterna_profile_paths()))

	dialog.exec()
	if not return_value:
		QMessageBox.critical(None, "Game data required", texts.SAVEGAME_REQUIRED)
		exit(1)
	return return_value

if __name__ == "__main__":
	logging.getLogger().setLevel(logging.DEBUG)

	qapp = QApplication(["EtternaGraph... 3?"])
	qapp.setWindowIcon(QIcon("assets/icon.ico"))

	try:
		config = backend.Config.load("etterna-graph-settings.json")
	except Exception as e:
		logging.warning(f"Couldn't load config: {e}")
		profile_paths = choose_profile(backend.detect_etterna_profiles())
		config = backend.Config(profile_paths)
	
	xml_stats = blocking_loading_bar(
		lambda *args: backend.load_xml_stats(config.paths.xml, *args),
		"Loading XML data..."
	)

	window = QMainWindow()
	window.resize(1280, 720)
	window.setCentralWidget(MainTabWidget(xml_stats))
	file_menu = window.menuBar().addMenu("File")
	file_menu.addAction("About", lambda: QMessageBox.about(None, "About", texts.ABOUT))
	file_menu.addAction("About Qt", lambda: QApplication.aboutQt())
	window.show()

	# start_time = time.time()
	# while start_time + 20 > time.time():
	qapp.exec()