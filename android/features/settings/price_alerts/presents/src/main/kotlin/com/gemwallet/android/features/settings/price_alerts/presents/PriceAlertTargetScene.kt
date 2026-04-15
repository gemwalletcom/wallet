package com.gemwallet.android.features.settings.price_alerts.presents

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.OutputTransformation
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowCircleDown
import androidx.compose.material.icons.filled.ArrowCircleUp
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.domains.price.PriceState
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.TabsBar
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.image.AssetIcon
import com.gemwallet.android.ui.components.list_item.Badge
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemTitleText
import com.gemwallet.android.ui.components.list_item.PriceInfo
import com.gemwallet.android.ui.components.parseMarkdownToAnnotatedString
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingLarge
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.features.settings.price_alerts.viewmodels.models.PriceAlertTargetError
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlertDirection
import com.wallet.core.primitives.PriceAlertNotificationType

private val tabs = listOf(
    PriceAlertNotificationType.Price,
    PriceAlertNotificationType.PricePercentChange,
)

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun PriceAlertTargetScene(
    value: TextFieldState = rememberTextFieldState(),
    type: PriceAlertNotificationType,
    direction: PriceAlertDirection,
    currency: Currency,
    currentPriceValue: Double,
    currentPriceFormatted: String,
    priceSuggestions: List<Pair<String, String>> = emptyList(),
    percentageSuggestions: List<Int> = listOf(5, 10, 15),
    asset: Asset? = null,
    assetPriceFormatted: String = "",
    assetPriceChangeFormatted: String = "",
    assetPriceState: PriceState = PriceState.None,
    error: PriceAlertTargetError?,
    onType: (PriceAlertNotificationType) -> Unit,
    onDirection: (PriceAlertDirection) -> Unit,
    onConfirm: () -> Unit,
    onCancel: () -> Unit,
) {
    val interactionSource = remember { MutableInteractionSource() }
    val focusRequester = remember { FocusRequester() }

    LaunchedEffect(Unit) {
        try {
            focusRequester.requestFocus()
        } catch (_: Throwable) {}
    }

    Scene(
        titleContent = {
            TabsBar(
                tabs = tabs,
                selected = type,
                onSelect = {
                    onType(it)
                    value.clearText()
                }
            ) { item ->
                Text(
                    stringResource(
                        when (item) {
                            PriceAlertNotificationType.Price ->  R.string.asset_price
                            PriceAlertNotificationType.PricePercentChange -> R.string.common_percentage
                            PriceAlertNotificationType.Auto -> R.string.common_no
                        }
                    ),
                )
            }
        },
        mainAction = {
            if (value.text.isEmpty()) {
                val suggestions = when (type) {
                    PriceAlertNotificationType.Price -> priceSuggestions
                    PriceAlertNotificationType.PricePercentChange -> percentageSuggestions.map { "$it%" to it.toString() }
                    else -> emptyList()
                }
                if (suggestions.isNotEmpty()) {
                    TabsBar(
                        tabs = suggestions,
                        selected = "" to "",
                        onSelect = { pair ->
                            value.edit { this.replace(0, this.length, pair.second) }
                        },
                        equalWidth = false,
                    ) { pair ->
                        Text(pair.first)
                    }
                }
            } else {
                MainActionButton(
                    title = stringResource(R.string.transfer_confirm),
                    enabled = error == null,
                    onClick = onConfirm,
                )
            }
        },
        onClose = onCancel,
    ) {
        Spacer(modifier = Modifier.size(paddingLarge * 2))
        LazyColumn(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(paddingSmall),
        ) {
            item {
                Text(
                    text = when (type) {
                        PriceAlertNotificationType.Auto -> ""
                        PriceAlertNotificationType.Price -> {
                            val inputPrice = try {
                                value.text.toString().toDouble()
                            } catch (_: Throwable) { 0.0 }
                            when {
                                inputPrice == 0.0 -> stringResource(R.string.price_alerts_set_alert_set_target_price)
                                inputPrice < currentPriceValue -> stringResource(R.string.price_alerts_set_alert_price_under)
                                inputPrice > currentPriceValue -> stringResource(R.string.price_alerts_set_alert_price_over)
                                else -> stringResource(R.string.price_alerts_set_alert_set_target_price)
                            }
                        }

                        PriceAlertNotificationType.PricePercentChange -> when (direction) {
                            PriceAlertDirection.Up -> stringResource(R.string.price_alerts_set_alert_price_increases_by)
                            PriceAlertDirection.Down -> stringResource(R.string.price_alerts_set_alert_price_decreases_by)
                        }
                    },
                    color = MaterialTheme.colorScheme.secondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
            item {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(paddingHalfSmall),
                ) {
                    Box(Modifier.weight(1f)) {
                        when (type) {
                            PriceAlertNotificationType.Price -> {
                                Text(
                                    modifier = Modifier.align(Alignment.CenterEnd),
                                    text = java.util.Currency.getInstance(currency.string).symbol,
                                    style = MaterialTheme.typography.displaySmall,
                                )
                            }
                            PriceAlertNotificationType.PricePercentChange -> {
                                Icon(
                                    modifier = Modifier.align(Alignment.CenterEnd).clickable {
                                        val direction = when (direction) {
                                            PriceAlertDirection.Up -> PriceAlertDirection.Down
                                            PriceAlertDirection.Down -> PriceAlertDirection.Up
                                        }
                                        onDirection(direction)
                                    },
                                    imageVector = when (direction) {
                                        PriceAlertDirection.Up -> Icons.Default.ArrowCircleUp
                                        PriceAlertDirection.Down -> Icons.Default.ArrowCircleDown
                                    },
                                    contentDescription = "",
                                    tint = when (direction) {
                                        PriceAlertDirection.Up -> MaterialTheme.colorScheme.tertiary
                                        PriceAlertDirection.Down -> MaterialTheme.colorScheme.error
                                    },
                                )
                            }
                            else -> {}
                        }
                    }
                    BasicTextField(
                        modifier = Modifier.width(IntrinsicSize.Min).focusRequester(focusRequester),
                        state = value,
                        lineLimits = TextFieldLineLimits.SingleLine,
                        textStyle = MaterialTheme.typography.displaySmall.copy(
                            textAlign = TextAlign.Center,
                            color = if (value.text.isEmpty()) MaterialTheme.colorScheme.secondary else MaterialTheme.colorScheme.onSurface
                        ),
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number, imeAction = ImeAction.Next),
                        interactionSource = interactionSource,
                        cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                        outputTransformation = OutputTransformation {
                            if (this.length == 0) {
                                this.append("0")
                            }
                        }
                    )
                    Box(Modifier.weight(1f)) {
                        if (type == PriceAlertNotificationType.PricePercentChange) {
                            Text(
                                text = "%",
                                style = MaterialTheme.typography.displaySmall,
                            )
                        }
                    }
                }
            }
            item {
                Text(
                    text = parseMarkdownToAnnotatedString("${stringResource(R.string.price_alerts_set_alert_current_price)} **$currentPriceFormatted**"),
                    color = MaterialTheme.colorScheme.secondary,
                    style = MaterialTheme.typography.bodyLarge,
                )
            }
            if (asset != null) {
                item {
                    ListItem(
                        listPosition = ListPosition.Single,
                        leading = { AssetIcon(asset) },
                        title = { ListItemTitleText(asset.name, titleBadge = { Badge(asset.symbol) }) },
                        subtitle = {
                            PriceInfo(
                                price = assetPriceFormatted,
                                changes = assetPriceChangeFormatted,
                                state = assetPriceState,
                                style = MaterialTheme.typography.bodyMedium,
                            )
                        },
                    )
                }
            }
        }
    }
}

@Preview
@Composable
fun PriceAlertTargetScenePricePreview() {
    WalletTheme {
        PriceAlertTargetScene(
            value = rememberTextFieldState(""),
            direction = PriceAlertDirection.Up,
            type = PriceAlertNotificationType.Price,
            currency = Currency.USD,
            currentPriceFormatted = "$901.80",
            currentPriceValue = 901.8,
            priceSuggestions = listOf("$850" to "850", "$950" to "950"),
            percentageSuggestions = listOf(3, 6, 9),
            error = null,
            onType = {},
            onDirection = {},
            onConfirm = {},
            onCancel = {},
        )
    }
}

@Preview
@Composable
fun PriceAlertTargetScenePercentagePreview() {
    WalletTheme {
        PriceAlertTargetScene(
            value = rememberTextFieldState(""),
            direction = PriceAlertDirection.Up,
            type = PriceAlertNotificationType.PricePercentChange,
            currency = Currency.USD,
            currentPriceFormatted = "$901.80",
            currentPriceValue = 901.8,
            priceSuggestions = listOf("$850" to "850", "$950" to "950"),
            percentageSuggestions = listOf(3, 6, 9),
            error = null,
            onType = {},
            onDirection = {},
            onConfirm = {},
            onCancel = {},
        )
    }
}