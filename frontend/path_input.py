from __future__ import annotations
from typing import *

from PyQt5.QtWidgets import QDialog, QGridLayout, QLineEdit, QPushButton, QLabel, QVBoxLayout
from PyQt5.QtWidgets import QDialogButtonBox, QWidget, QFileDialog, QMessageBox
from PyQt5.Qt import QIcon, QApplication, QStyle

import backend, texts


# Returns None if user quitted the dialog
def request_etterna_profile_paths() -> Optional[backend.EtternaProfilePaths]:
	dialog = QDialog()
	vbox_layout = QVBoxLayout()
	dialog.setLayout(vbox_layout)

	vbox_layout.addWidget(QLabel("Please enter all of the paths:"))

	input_grid = QWidget()
	grid_layout = QGridLayout()
	input_grid.setLayout(grid_layout)
	vbox_layout.addWidget(input_grid)

	button_box = QDialogButtonBox(QDialogButtonBox.Ok | QDialogButtonBox.Cancel)
	vbox_layout.addWidget(button_box)

	def path_input(name: str, directory: bool, preamble: Optional[str] = None) -> QLineEdit:
		line_edit = QLineEdit()
		button = QPushButton()

		if directory:
			button.setIcon(QIcon.fromTheme("folder-open", QApplication.style().standardIcon(QStyle.SP_DirIcon)))
		else:
			button.setIcon(QIcon.fromTheme("document-open", QApplication.style().standardIcon(QStyle.SP_FileIcon)))
		
		def press_callback() -> None:
			if directory:
				if preamble: QMessageBox.information(None, "How to use", preamble)
				path = QFileDialog.getExistingDirectory(caption=f"Select your {name} directory")
			else:
				path, filter = QFileDialog.getOpenFileName(caption=f"Select your {name} file")
			
			if path:
				line_edit.setText(path)
		button.pressed.connect(press_callback)

		row = grid_layout.rowCount()
		grid_layout.addWidget(QLabel(name), row, 0)
		grid_layout.addWidget(line_edit, row, 1)
		grid_layout.addWidget(button, row, 2)

		return line_edit

	xml_input = path_input("Etterna.xml", directory=False, preamble=None)
	replays_input = path_input("ReplaysV2", directory=True, preamble=texts.REPLAYS_CHOOSER_INFO_MSG)
	songs_input = path_input("Songs", directory=True, preamble=texts.SONGS_ROOT_CHOOSER_INFO_MSG)

	return_value = None # will be set when OK is clicked
	def ok_was_clicked() -> None:
		profile_paths = backend.EtternaProfilePaths(
			xml=xml_input.text(),
			replays_dir=replays_input.text(),
			songs_dir=songs_input.text(),
		)

		if profile_paths.exists():
			nonlocal return_value
			return_value = profile_paths
			dialog.accept()
		else:
			QMessageBox.critical(None, "Missing inputs", "Please fill in all of the fields correctly")

	button_box.rejected.connect(lambda: dialog.reject())
	button_box.accepted.connect(ok_was_clicked)

	dialog.exec()
	return return_value