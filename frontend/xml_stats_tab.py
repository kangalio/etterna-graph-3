from __future__ import annotations
from typing import *

from datetime import datetime, date

from PyQt5.QtWidgets import QPushButton, QApplication, QMainWindow, QTabWidget, QLabel, QMessageBox
from PyQt5.QtWidgets import QWidget, QGridLayout, QVBoxLayout, QRadioButton, QFrame, QDialog
from PyQt5.QtWidgets import QDialogButtonBox, QLineEdit, QStyle, QHBoxLayout, QComboBox

import backend, globals
from plot_wrapper import PlotWrapper, ScatterPlotItem, PlotItem, LinePlotItem, LinkGroup
from loading_bar import blocking_loading_bar


# Color for the all-grades-considered accuracy rating over time
ALL_GRADES_COLOR = "ffffff"

def vertical_separator() -> QWidget:
	line = QFrame()
	line.setFrameShape(QFrame.VLine)
	line.setFrameShadow(QFrame.Sunken)
	return line

T = TypeVar("T")
def find_rating_at(target_dt: datetime, ratings: List[Tuple[datetime, T]]) -> Optional[T]:
	try:
		return next(rating for dt, rating in reversed(ratings) if dt <= target_dt)
	except StopIteration:
		return None # Cursor is before first rating

class SkillsetsOverTime(QWidget):
	def __init__(self, stats: backend.XmlStats, link_group: LinkGroup):
		super().__init__()
		self._stats = stats
		self._acc_rating_over_time: Optional[Any] = None # Will be calculated on-demand
		self._link_group = link_group

		self._layout = QVBoxLayout()
		self.setLayout(self._layout)

		self._plot = QWidget() # This dummy will be replaced by setup function later
		self._layout.addWidget(self._plot)

		bottom_pane = QWidget()
		sub_layout = QHBoxLayout()
		bottom_pane.setLayout(sub_layout)
		self._layout.addWidget(bottom_pane)

		self._cursor_pos_span = QLabel()
		self._cursor_pos_span.setWordWrap(True)
		sub_layout.addWidget(self._cursor_pos_span)

		sub_layout.addWidget(vertical_separator())

		display_options = QComboBox()
		display_options.addItem("Show all scores")
		display_options.addItem("Show AAA-/AAAA-only")
		display_options.currentIndexChanged.connect(self._current_index_changed)
		sub_layout.addWidget(display_options)

		self._current_index_changed(0) # Trigger first render

	def _current_index_changed(self, index: int):
		if index == 0:
			new_plot = self._setup_skillsets()
		elif index == 1:
			new_plot = self._setup_acc_rating()
		else:
			print(f"Warning: unknown dropdown index {index}. Ignoring")
			return
		
		self._layout.replaceWidget(self._plot, new_plot)
		self._plot = new_plot
		self._layout.invalidate() # Causes glitches otherwise

	def _crosshair_moved_skillsets(self, cursor_x: datetime) -> None:
		rating = find_rating_at(cursor_x, self._stats.skillsets_over_time) or [0, 0, 0, 0, 0, 0, 0, 0]

		text = f"{cursor_x.date()}: "
		for i in range(8):
			color = globals.SKILLSET_COLORS_8[i]
			name = globals.SKILLSET_NAMES_8[i]
			text_color = globals.SKILLSET_CONTRASTING_TEXT_COLORS_8[i]
			text += f'<span style="background-color:#{color}; color:#{text_color}">{name}: <b>{rating[i]:.2f}</b> </span>'
		
		self._cursor_pos_span.setText(text)
	
	def _setup_skillsets(self) -> PlotWrapper:
		plot_items = []
		for i in range(8):
			plot_items.append(PlotItem(
				data=LinePlotItem(points=[(dt, rating[i]) for dt, rating in self._stats.skillsets_over_time]),
				color=globals.SKILLSET_COLORS_8[i],
				legend_name=globals.SKILLSET_NAMES_8[i],
			))
		# Make overall line thick
		plot_items[0].data.width *= 3 # type: ignore
		
		return PlotWrapper(
			item=plot_items,
			title="Skillsets over time",
			datetime_x_axis=True,
			show_x_crosshair=True,
			crosshair_move_callback=self._crosshair_moved_skillsets,
			link_group=self._link_group,
		)
	
	def _crosshair_moved_acc(self, cursor_x: datetime) -> None:
		if not self._acc_rating_over_time:
			print("Warning: acc_rating_over_time not set in mouse move handler; skipping")
			return

		normal_rating = find_rating_at(cursor_x, self._acc_rating_over_time.normal) or 0.0
		aaa_rating = find_rating_at(cursor_x, self._acc_rating_over_time.aaa) or 0.0
		aaaa_rating = find_rating_at(cursor_x, self._acc_rating_over_time.aaaa) or 0.0

		text = f"{cursor_x.date()}: " +\
			f'<span style="background-color:#{ALL_GRADES_COLOR}; color:#000000">All: <b>{normal_rating:.2f}</b> </span>' +\
			f'<span style="background-color:#{globals.AAA_COLOR}; color:#000000">AAA: <b>{aaa_rating:.2f}</b> </span>' +\
			f'<span style="background-color:#{globals.AAAA_COLOR}; color:#000000">AAAA: <b>{aaaa_rating:.2f}</b> </span>'
		
		self._cursor_pos_span.setText(text)

	def _setup_acc_rating(self) -> PlotWrapper:
		if not self._acc_rating_over_time:
			self._acc_rating_over_time = blocking_loading_bar(
				lambda *args: backend.calculate_acc_rating_over_time(self._stats, *args),
				"Calculating accuracy ratings",
			)
		
		def make_plot_item(ratings: List[Tuple[date, float]], color: str, name: str) -> PlotItem:
			return PlotItem(
				data=LinePlotItem(
					points=ratings,
					width=3, # we have 3 distinct lines, might as well make them thicc
				),
				color=color,
				legend_name=name,
			)
		
		return PlotWrapper(
			item=[
				make_plot_item(self._acc_rating_over_time.normal, ALL_GRADES_COLOR, "All scores"),
				make_plot_item(self._acc_rating_over_time.aaa, globals.AAA_COLOR, "Only AAA"),
				make_plot_item(self._acc_rating_over_time.aaaa, globals.AAAA_COLOR, "Only AAAA"),
			],
			title="Skillsets over time",
			datetime_x_axis=True,
			show_x_crosshair=True,
			crosshair_move_callback=self._crosshair_moved_acc,
			link_group=self._link_group,
		)

class ScoreRatingOverTime(PlotWrapper):
	def __init__(self, stats: backend.XmlStats, link_group: LinkGroup):
		super().__init__(
			item=[
				PlotItem(
					data=ScatterPlotItem(dots=stats.ssr_over_time.aaaa_and_above),
					color=globals.AAAA_COLOR, legend_name="AAAA and above",
				),
				PlotItem(
					data=ScatterPlotItem(dots=stats.ssr_over_time.aaa),
					color=globals.AAA_COLOR, legend_name="AAA",
				),
				PlotItem(
					data=ScatterPlotItem(dots=stats.ssr_over_time.aa),
					color=globals.AA_COLOR, legend_name="AA",
				),
				PlotItem(
					data=ScatterPlotItem(dots=stats.ssr_over_time.a),
					color=globals.A_COLOR, legend_name="A",
				),
				PlotItem(
					data=ScatterPlotItem(dots=stats.ssr_over_time.b_and_below),
					color=globals.B_COLOR, legend_name="B and below",
				),
			],
			title="Score rating over time",
			datetime_x_axis=True,
			show_x_crosshair=True,
			link_group=link_group,
		)

class AccuracyOverTime(PlotWrapper):
	def __init__(self, stats: backend.XmlStats, link_group: LinkGroup):
		super().__init__(
			item=PlotItem(data=ScatterPlotItem(dots=stats.acc_over_time), color="#1f77b4"),
			title="Accuracy over time",
			datetime_x_axis=True,
			show_x_crosshair=True,
			link_group=link_group,
		)

class XmlStatsTab(QWidget):
	def __init__(self, stats: backend.XmlStats):
		super().__init__()

		layout = QGridLayout()
		self.setLayout(layout)

		link_group = LinkGroup()

		layout.addWidget(ScoreRatingOverTime(stats, link_group), 0, 0)
		layout.addWidget(SkillsetsOverTime(stats, link_group), 0, 1)
		layout.addWidget(AccuracyOverTime(stats, link_group), 1, 0)
