from __future__ import annotations
from typing import *
from returns.primitives.hkt import Kind1, kinded

from datetime import datetime
from dataclasses import dataclass

import pyqtgraph as pg
from PyQt5.QtCore import QPointF

import globals


PRIMARY_CROSSHAIR_PEN = pg.mkPen(0.5)
SECONDARY_CROSSHAIR_PEN = pg.mkPen(0.3)

A = TypeVar("A")
B = TypeVar("B")
def _transpose_tuples(tuples: Iterable[Tuple[A, B]]) -> Tuple[List[A], List[B]]:
	# dunno if this can be done efficiently or if it's even worth
	# remember that Iterables may only be iterated once
	a, b = zip(*tuples)
	return (list(a), list(b))

X = TypeVar("X", datetime, float) # possible types for x coordinates

@dataclass
class ScatterPlotItem(Generic[X]):
	dots: List[Tuple[X, float]]

# Doesn't interpolate by default
@dataclass
class LinePlotItem(Generic[X]):
	points: List[Tuple[X, float]]
	width: int = 1

@dataclass
class PlotItem(Generic[X]):
	data: Union[ScatterPlotItem[X], LinePlotItem[X]]
	color: str
	legend_name: Optional[str] = None

class PlotWrapper(pg.PlotWidget, Generic[X]):
	def __init__(self,
		item: Union[Iterable[PlotItem[X]], PlotItem[X]],
		title: str,
		datetime_x_axis: bool = False,
		show_x_crosshair: bool = False,
		crosshair_move_callback: Callable[[X], None] = lambda x: None,
		link_group: LinkGroup = None,
	):
		self._datetime_x_axis = datetime_x_axis
		
		# These are global config options, but there's no neat place to put them so we just
		# repeatedly initialize them to the same value in here
		pg.setConfigOption("background", globals.BG_COLOR)
		pg.setConfigOption("foreground", globals.TEXT_COLOR)

		axes = {}
		if datetime_x_axis:
			axes["bottom"] = pg.DateAxisItem(orientation="bottom")
		super().__init__(axisItems = axes)
		
		plot: pg.PlotItem = self.getPlotItem()
		plot.setTitle(title)
		plot.showGrid(x=True, y=True, alpha=0.15)
		
		self._setup_items_and_legend([item] if isinstance(item, PlotItem) else list(item))
		self._setup_crosshair(show_x_crosshair, crosshair_move_callback, link_group)
	
	def _setup_items_and_legend(self, item_specs: List[PlotItem[X]]):
		legend = pg.LegendItem()
		items_to_add_to_plot = []
		for item_spec in item_specs:
			color = pg.mkColor(item_spec.color)
			
			if isinstance(item_spec.data, ScatterPlotItem):
				# Extract list of coordinate tuples into tuple of coordinate lists
				x, y = _transpose_tuples(item_spec.data.dots)

				# semi-transparent scatter dots are nice :)
				color.setAlphaF(0.8)

				# PyQtGraph's datetime axis expects timestamps
				if self._datetime_x_axis:
					x = [x_val.timestamp() for x_val in x] # type: ignore
				
				item = pg.ScatterPlotItem(x, y, pen=None, size=8, brush=color)
			elif isinstance(item_spec.data, LinePlotItem):
				# Extract list of coordinate tuples into tuple of coordinate lists
				x, y = _transpose_tuples(item_spec.data.points)

				# PyQtGraph's datetime axis expects timestamps
				if self._datetime_x_axis:
					x = [x_val.timestamp() for x_val in x] # type: ignore
				
				pen = pg.mkPen(color, width=item_spec.data.width)
				item = pg.PlotCurveItem(x, y, pen=pen, stepMode="left")
			
			items_to_add_to_plot.append(item)
			if item_spec.legend_name:
				legend.addItem(item, item_spec.legend_name)
		
		# Draw in reverse so the first supplied item is on top
		for item in reversed(items_to_add_to_plot):
			self.getPlotItem().addItem(item)
		
		# Show legend only if there were items with an associated legend name
		if len(legend.items) >= 1:
			legend.setBrush(globals.LEGEND_BG_COLOR)
			legend.setPen(globals.BORDER_COLOR)
			legend.setParentItem(self.getPlotItem())
			# Anchor the item's edge at the parent's edge with a certain offset
			legend.anchor(itemPos=(0, 0), parentPos=(0, 0), offset=(45, 45))

	def _setup_crosshair(self,
		show_x_crosshair: bool,
		crosshair_move_callback: Callable[[X], None] = lambda x: None,
		link_group: LinkGroup = None,
	) -> None:
		self._crosshair_move_callback: Callable[[X], None] = crosshair_move_callback
		self._link_group = link_group or LinkGroup()
		self._link_group._plots.append(self)

		plot: pg.PlotItem = self.getPlotItem()

		if show_x_crosshair:
			self._x_crosshair = pg.InfiniteLine(angle=90, movable=False)
			plot.addItem(self._x_crosshair, ignoreBounds=True)
		else:
			self._x_crosshair = None

		def mouse_moved(pos: QPointF):
			if not plot.sceneBoundingRect().contains(pos): return
			pos = plot.vb.mapSceneToView(pos)

			for linked_plot in self._link_group._plots:
				linked_plot._update_crosshair(pos.x(), primary=(linked_plot == self))
		plot.scene().sigMouseMoved.connect(mouse_moved)

	# Called from link group
	def _update_crosshair(self, x: float, primary: bool) -> None:
		if self._x_crosshair:
			self._x_crosshair.setPen(PRIMARY_CROSSHAIR_PEN if primary else SECONDARY_CROSSHAIR_PEN)
			self._x_crosshair.setPos(x)
		
		x: X = datetime.fromtimestamp(x) if self._datetime_x_axis else x # type: ignore
		(self._crosshair_move_callback)(x)


class LinkGroup:
	def __init__(self):
		self._plots = []