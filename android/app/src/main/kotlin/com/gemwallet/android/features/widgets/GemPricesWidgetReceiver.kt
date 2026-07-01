package com.gemwallet.android.features.widgets

import android.content.Context
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.GlanceAppWidgetReceiver

class GemPricesWidgetReceiver : GlanceAppWidgetReceiver() {

    override val glanceAppWidget: GlanceAppWidget = PricesWidget()

    override fun onEnabled(context: Context) {
        super.onEnabled(context)
        WidgetPriceSyncWorker.schedule(context)
    }

    override fun onDisabled(context: Context) {
        super.onDisabled(context)
        WidgetPriceSyncWorker.cancel(context)
    }
}
