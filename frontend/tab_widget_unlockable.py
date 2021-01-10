from __future__ import annotations
from typing import *

from PyQt5.QtWidgets import QTabWidget, QTabBar, QWidget


class TabWidgetUnlockable(QTabWidget):
	def __init__(self):
		super().__init__()
		
		self._last_clicked_tab = -1
		self._prev_tab = -1
		self._ignore_post_tab_change = False

		def user_clicked_tab(requested_tab: int) -> None:
			self._prev_tab = self.currentIndex()
			self._last_clicked_tab = requested_tab

		def tab_changed(switched_to_tab_index: int) -> None:
			if switched_to_tab_index != self._last_clicked_tab:
				# this tab change was not initiated by the user
				return

			# if an unlock callback is registered, this tab is locked and we need to intervene
			unlock_callback = getattr(self.currentWidget(), "_unlock_callback", None)
			if unlock_callback:
				# quickly switch back to prev tab to not show the uninitialized contents of the
				# unlocked tab
				self._ignore_post_tab_change = True
				self.setCurrentIndex(self._prev_tab)
				self._ignore_post_tab_change = False

				new_widget = (unlock_callback)()
				if new_widget:
					# insert new contents and then switch to the new tab
					self._ignore_post_tab_change = True
					self.replaceTabContents(switched_to_tab_index, new_widget)
					self.setCurrentIndex(switched_to_tab_index)
					self._ignore_post_tab_change = False
				else:
					# callback gave no replacement widget, so this tab remains locked
					pass
		
		self.tabBarClicked.connect(user_clicked_tab)
		self.currentChanged.connect(tab_changed)
	
	def addUnlockableTab(self, widget_callback: Callable[[], Optional[QWidget]], label: str) -> int:
		empty_widget = QWidget()
		empty_widget._unlock_callback = widget_callback # piggyback our own state onto the widget
		return self.addTab(empty_widget, label)
	
	## This function will unlock the tab
	## Qt is a shitfest, therefore this function is implemented using removeTab and insertTab.
	## Beware of the potential false positive signal triggers (e.g. currentChanged)
	# why the FUCK do I have to reimplement this myself and why the FUCK is it so clunky
	# and this implementation will break the second Qt adds a new attribute BUT THERES NO FUCKING
	# BETTER WAY AAAAAAAAAAAAAAAAAAGLURFSIGUIHLRSILUHSGRHUILSGRHUILSGRUH
	def replaceTabContents(self, index: int, new_widget: QWidget) -> None:
		# get attributes of old tab
		tabButtonLeft = self.tabBar().tabButton(index, QTabBar.ButtonPosition.LeftSide)
		tabButtonRight = self.tabBar().tabButton(index, QTabBar.ButtonPosition.RightSide)
		tabData = self.tabBar().tabData(index)
		tabIcon = self.tabBar().tabIcon(index)
		tabText = self.tabBar().tabText(index)
		tabTextColor = self.tabBar().tabTextColor(index)
		tabToolTip = self.tabBar().tabToolTip(index)
		tabWhatsThis = self.tabBar().tabWhatsThis(index)

		# remove old tab and insert new one
		self.removeTab(index)
		self.insertTab(index, new_widget, tabText)

		# add attributes of old tab onto new tab to make it look like just the contents have changed
		self.tabBar().setTabButton(index, QTabBar.ButtonPosition.LeftSide, tabButtonLeft)
		self.tabBar().setTabButton(index, QTabBar.ButtonPosition.RightSide, tabButtonRight)
		self.tabBar().setTabData(index, tabData)
		self.tabBar().setTabIcon(index, tabIcon)
		self.tabBar().setTabText(index, tabText)
		self.tabBar().setTabTextColor(index, tabTextColor)
		self.tabBar().setTabToolTip(index, tabToolTip)
		self.tabBar().setTabWhatsThis(index, tabWhatsThis)

	
