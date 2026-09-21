package com.gemwallet.android.features.widgets

import android.content.Context
import androidx.glance.appwidget.updateAll
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.NetworkType
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import com.gemwallet.android.data.services.gemstone.di.WidgetEntryPoint
import dagger.hilt.android.EntryPointAccessors
import java.util.concurrent.TimeUnit

class WidgetPriceSyncWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {

    override suspend fun doWork(): Result = try {
        PricesWidget().updateAll(applicationContext)
        Result.success()
    } catch (_: Throwable) {
        Result.retry()
    }

    companion object {
        private const val WORK_NAME = "widget_price_sync"

        fun schedule(context: Context) {
            val widgetService = EntryPointAccessors.fromApplication(context, WidgetEntryPoint::class.java).widgetService()
            val request = PeriodicWorkRequestBuilder<WidgetPriceSyncWorker>(
                widgetService.refreshIntervalSeconds().toLong(),
                TimeUnit.SECONDS,
            )
                .setConstraints(
                    Constraints.Builder()
                        .setRequiredNetworkType(NetworkType.CONNECTED)
                        .build(),
                )
                .build()
            WorkManager.getInstance(context).enqueueUniquePeriodicWork(
                WORK_NAME,
                ExistingPeriodicWorkPolicy.KEEP,
                request,
            )
        }

        fun cancel(context: Context) {
            WorkManager.getInstance(context).cancelUniqueWork(WORK_NAME)
        }
    }
}
