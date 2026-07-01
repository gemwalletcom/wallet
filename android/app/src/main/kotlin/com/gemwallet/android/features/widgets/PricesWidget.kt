package com.gemwallet.android.features.widgets

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
import com.gemwallet.android.data.repositories.di.WidgetEntryPoint
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.Currency
import dagger.hilt.android.EntryPointAccessors
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext

private data class WidgetAsset(val info: AssetInfo, val icon: Bitmap?)

class PricesWidget : GlanceAppWidget() {

    override suspend fun provideGlance(
        context: Context,
        id: GlanceId
    ) {
        val assetsRepository = EntryPointAccessors.fromApplication(context, WidgetEntryPoint::class.java).assetsRepository()
        val sessionRepository = EntryPointAccessors.fromApplication(context, WidgetEntryPoint::class.java).sessionRepository()
        val noData = context.getString(R.string.errors_no_data_available)
        val items = try {
            val currency = sessionRepository.session().firstOrNull()?.currency ?: Currency.USD
            loadItems(context, assetsRepository.getWidgetTokens(currency))
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

    private suspend fun loadItems(context: Context, assets: List<AssetInfo>): List<WidgetAsset> = coroutineScope {
        assets.map { asset ->
            async { WidgetAsset(asset, loadIcon(context, asset.id().getIconUrl())) }
        }.awaitAll()
    }

    private suspend fun loadIcon(context: Context, url: String): Bitmap? = withContext(Dispatchers.IO) {
        runCatching {
            val request = ImageRequest.Builder(context).data(url).build()
            (context.imageLoader.execute(request) as? SuccessResult)?.image?.toBitmap()
        }.getOrNull()
    }

    @Composable
    private fun Assets(items: List<WidgetAsset>) {
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
private fun AssetItem(item: WidgetAsset) {
    val asset = item.info
    Row(
        modifier = GlanceModifier
            .fillMaxWidth()
            .height(72.dp)
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
            WidgetTitleText(asset.asset.name)
            Spacer(GlanceModifier.size(paddingHalfSmall))
            WidgetSubtitleText(asset.asset.symbol)
        }
        asset.price?.let {
            val percentageColor = when {
                it.price.priceChangePercentage24h < 0 -> Color(0xFFF84E4E)
                it.price.priceChangePercentage24h > 0 -> Color(0xFF06BE92)
                else -> Color(0xFF808d99)
            }
            Column(
                modifier = GlanceModifier.defaultWeight(),
                horizontalAlignment = Alignment.End,
            ) {
                WidgetTitleText(CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = it.currency).string(it.price.price))
                Spacer(GlanceModifier.size(paddingHalfSmall))
                WidgetSubtitleText(it.price.priceChangePercentage24h.formatAsPercentage(), percentageColor)
            }
        }
    }
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
