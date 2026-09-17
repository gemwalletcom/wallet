package com.gemwallet.android.features.widgets

import com.gemwallet.android.ui.components.image.iconModel
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.glance.GlanceId
import androidx.glance.GlanceModifier
import androidx.glance.GlanceTheme
import androidx.glance.Image
import androidx.glance.ImageProvider
import androidx.glance.action.clickable
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.action.actionStartActivity
import androidx.glance.appwidget.provideContent
import androidx.glance.background
import androidx.glance.layout.Alignment
import androidx.glance.layout.Box
import androidx.glance.layout.Column
import androidx.glance.layout.Row
import androidx.glance.layout.Spacer
import androidx.glance.layout.fillMaxSize
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.height
import androidx.glance.layout.padding
import androidx.glance.layout.size
import androidx.glance.text.FontWeight
import androidx.glance.text.Text
import androidx.glance.text.TextAlign
import androidx.glance.text.TextDefaults.defaultTextStyle
import androidx.glance.unit.ColorProvider
import coil3.imageLoader
import coil3.request.ImageRequest
import coil3.request.SuccessResult
import coil3.toBitmap
import com.gemwallet.android.MainActivity
import com.gemwallet.android.data.services.gemstone.di.WidgetEntryPoint
import com.gemwallet.android.ui.R
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.GemWidgetCoin
import uniffi.gemstone.GemWidgetSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import dagger.hilt.android.EntryPointAccessors
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.withContext

private val priceRowHeight = 72.dp

private data class WidgetCoin(val coin: GemWidgetCoin, val icon: Bitmap?)

class PricesWidget : GlanceAppWidget() {

    override suspend fun provideGlance(
        context: Context,
        id: GlanceId
    ) {
        val entryPoint = EntryPointAccessors.fromApplication(context, WidgetEntryPoint::class.java)
        val noData = context.getString(R.string.errors_no_data_available)
        val items = try {
            val currency = entryPoint.preferencesService().getCurrency().toPrimitives().string
            loadItems(context, withContext(Dispatchers.IO) { entryPoint.widgetService().coins(GemWidgetSize.MEDIUM, currency) })
        } catch (_: Throwable) {
            emptyList()
        }

        provideContent {
            val launchAppIntent = Intent(context, MainActivity::class.java)
            GlanceTheme {
                Box(
                    modifier = GlanceModifier
                        .fillMaxSize()
                        .background(MaterialTheme.colorScheme.background)
                        .clickable(actionStartActivity(launchAppIntent)),
                ) {
                    if (items.isNotEmpty()) {
                        Assets(items)
                    } else {
                        Text(
                            modifier = GlanceModifier.fillMaxSize().padding(paddingDefault),
                            text = noData,
                            style = defaultTextStyle.copy(textAlign = TextAlign.Center),
                        )
                    }
                }
            }
        }
    }

    private suspend fun loadItems(context: Context, coins: List<GemWidgetCoin>): List<WidgetCoin> = coroutineScope {
        coins.map { coin ->
            async { WidgetCoin(coin, loadIcon(context, coin.assetId.toAssetId()?.iconModel())) }
        }.awaitAll()
    }

    private suspend fun loadIcon(context: Context, model: Any?): Bitmap? = withContext(Dispatchers.IO) {
        model ?: return@withContext null
        runCatching {
            val request = ImageRequest.Builder(context).data(model).build()
            (context.imageLoader.execute(request) as? SuccessResult)?.image?.toBitmap()
        }.getOrNull()
    }

    @Composable
    private fun Assets(items: List<WidgetCoin>) {
        Column(
            modifier = GlanceModifier.fillMaxSize(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            items.forEach {
                AssetItem(it)
            }
        }
    }

    companion object {
        init {
            System.loadLibrary("gemstone")
        }
    }
}

@Composable
private fun AssetItem(item: WidgetCoin) {
    val coin = item.coin
    Row(
        modifier = GlanceModifier
            .fillMaxWidth()
            .height(priceRowHeight)
            .padding(horizontal = paddingDefault, vertical = paddingSmall),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box() {
            item.icon?.let {
                Image(
                    ImageProvider(it),
                    contentDescription = ""
                )
            }
        }
        Spacer(GlanceModifier.size(paddingDefault))
        Column(
            modifier = GlanceModifier.defaultWeight()
        ) {
            WidgetTitleText(coin.name)
            Spacer(GlanceModifier.size(paddingHalfSmall))
            WidgetSubtitleText(coin.symbol)
        }
        Column(
            modifier = GlanceModifier.defaultWeight(),
            horizontalAlignment = Alignment.End,
        ) {
            WidgetTitleText(coin.price.text())
            Spacer(GlanceModifier.size(paddingHalfSmall))
            WidgetSubtitleText(coin.change.text(), coin.change.tone.widgetColor())
        }
    }
}

private fun GemValueTone.widgetColor(): Color = when (this) {
    GemValueTone.POSITIVE -> Color(0xFF06BE92)
    GemValueTone.NEGATIVE -> Color(0xFFF84E4E)
    GemValueTone.NEUTRAL, GemValueTone.PLAIN -> Color(0xFF808d99)
}

@Composable
private fun WidgetTitleText(text: String) {
    Text(
        modifier = GlanceModifier,
        text = text,
        maxLines = 1,
        style = defaultTextStyle.copy(fontSize = 16.sp, fontWeight = FontWeight.Medium),
    )
}

@Composable
private fun WidgetSubtitleText(text: String, color: Color = Color.Black) {
    Text(
        modifier = GlanceModifier,
        text = text,
        maxLines = 1,
        style = defaultTextStyle.copy(fontSize = 14.sp, fontWeight = FontWeight.Normal, color = ColorProvider(color)),
    )
}
