package com.gemwallet.android.features.price_alerts.presents

import androidx.annotation.StringRes
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
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
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
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.TabsBar
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.fields.requestFocusIfAttached
import com.gemwallet.android.ui.components.list_item.AssetListItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingLarge
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlertDirection
import com.wallet.core.primitives.PriceAlertNotificationType
import uniffi.gemstone.GemAmountSymbolPlacement
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemPriceAlertInput
import uniffi.gemstone.GemPriceAlertSymbol

private val tabs = listOf(
    PriceAlertNotificationType.Price,
    PriceAlertNotificationType.PricePercentChange,
)

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun SetPriceAlertScene(
    value: TextFieldState = rememberTextFieldState(),
    type: PriceAlertNotificationType,
    direction: PriceAlertDirection,
    @StringRes prompt: Int,
    input: GemPriceAlertInput,
    currentPriceText: String,
    priceSuggestions: List<Pair<String, String>> = emptyList(),
    percentageSuggestions: List<Pair<String, String>> = emptyList(),
    assetRow: GemAssetItemRow? = null,
    buttonState: ButtonState,
    snackbar: SnackbarHostState? = null,
    onType: (PriceAlertNotificationType) -> Unit,
    onDirection: (PriceAlertDirection) -> Unit,
    onConfirm: () -> Unit,
    onCancel: () -> Unit,
) {
    val interactionSource = remember { MutableInteractionSource() }
    val focusRequester = remember { FocusRequester() }

    LaunchedEffect(Unit) {
        focusRequester.requestFocusIfAttached()
    }

    Scene(
        snackbar = snackbar,
        titleContent = {
            TabsBar(
                tabs = tabs,
                selected = type,
                onSelect = {
                    onType(it)
                    value.clearText()
                },
            ) { item ->
                Text(
                    stringResource(
                        when (item) {
                            PriceAlertNotificationType.Price -> R.string.asset_price
                            PriceAlertNotificationType.PricePercentChange -> R.string.common_percentage
                            PriceAlertNotificationType.Auto -> R.string.common_no
                        },
                    ),
                )
            }
        },
        mainAction = {
            if (value.text.isEmpty()) {
                val suggestions = when (type) {
                    PriceAlertNotificationType.Price -> priceSuggestions
                    PriceAlertNotificationType.PricePercentChange -> percentageSuggestions
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
                    state = buttonState,
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
                    text = stringResource(prompt),
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
                        val directionButton = input.directionButton
                        if (directionButton != null) {
                            Icon(
                                modifier = Modifier.align(Alignment.CenterEnd).clickable {
                                    val direction = when (direction) {
                                        PriceAlertDirection.Up -> PriceAlertDirection.Down
                                        PriceAlertDirection.Down -> PriceAlertDirection.Up
                                    }
                                    onDirection(direction)
                                },
                                imageVector = when (directionButton) {
                                    uniffi.gemstone.PriceAlertDirection.UP -> AppIcons.ArrowCircleUp
                                    uniffi.gemstone.PriceAlertDirection.DOWN -> AppIcons.ArrowCircleDown
                                },
                                contentDescription = "",
                                tint = when (directionButton) {
                                    uniffi.gemstone.PriceAlertDirection.UP -> MaterialTheme.colorScheme.tertiary
                                    uniffi.gemstone.PriceAlertDirection.DOWN -> MaterialTheme.colorScheme.error
                                },
                            )
                        } else if (input.placement == GemAmountSymbolPlacement.LEADING) {
                            Text(
                                modifier = Modifier.align(Alignment.CenterEnd),
                                text = input.symbol.text(),
                                style = MaterialTheme.typography.displaySmall,
                            )
                        }
                    }
                    BasicTextField(
                        modifier = Modifier.width(IntrinsicSize.Min).focusRequester(focusRequester),
                        state = value,
                        lineLimits = TextFieldLineLimits.SingleLine,
                        textStyle = MaterialTheme.typography.displaySmall.copy(
                            textAlign = TextAlign.Center,
                            color = if (value.text.isEmpty()) MaterialTheme.colorScheme.secondary else MaterialTheme.colorScheme.onSurface,
                        ),
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal, imeAction = ImeAction.Next),
                        interactionSource = interactionSource,
                        cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                        outputTransformation = OutputTransformation {
                            if (this.length == 0) {
                                this.append(input.placeholder)
                            }
                        },
                    )
                    Box(Modifier.weight(1f)) {
                        if (input.placement == GemAmountSymbolPlacement.TRAILING) {
                            Text(
                                text = input.symbol.text(),
                                style = MaterialTheme.typography.displaySmall,
                            )
                        }
                    }
                }
            }
            item {
                Text(
                    text = currentPriceText,
                    color = MaterialTheme.colorScheme.secondary,
                    style = MaterialTheme.typography.bodyLarge,
                )
            }
            if (assetRow != null) {
                item {
                    AssetListItem(row = assetRow, listPosition = ListPosition.Single)
                }
            }
        }
    }
}

@Preview
@Composable
fun SetPriceAlertScenePricePreview() {
    WalletTheme {
        SetPriceAlertScene(
            value = rememberTextFieldState(""),
            direction = PriceAlertDirection.Up,
            type = PriceAlertNotificationType.Price,
            input = GemPriceAlertInput("0", GemPriceAlertSymbol.Currency(uniffi.gemstone.Currency.USD), GemAmountSymbolPlacement.LEADING, null),
            currentPriceText = "Current price $901.80",
            prompt = R.string.price_alerts_set_alert_price_over,
            priceSuggestions = listOf("$850" to "850", "$950" to "950"),
            percentageSuggestions = listOf("3%" to "3", "6%" to "6", "9%" to "9"),
            buttonState = ButtonState.Enabled,
            onType = {},
            onDirection = {},
            onConfirm = {},
            onCancel = {},
        )
    }
}

@Preview
@Composable
fun SetPriceAlertScenePercentagePreview() {
    WalletTheme {
        SetPriceAlertScene(
            value = rememberTextFieldState(""),
            direction = PriceAlertDirection.Up,
            type = PriceAlertNotificationType.PricePercentChange,
            input = GemPriceAlertInput("5", GemPriceAlertSymbol.Percent, GemAmountSymbolPlacement.TRAILING, uniffi.gemstone.PriceAlertDirection.UP),
            currentPriceText = "Current price $901.80",
            prompt = R.string.price_alerts_set_alert_price_over,
            priceSuggestions = listOf("$850" to "850", "$950" to "950"),
            percentageSuggestions = listOf("3%" to "3", "6%" to "6", "9%" to "9"),
            buttonState = ButtonState.Enabled,
            onType = {},
            onDirection = {},
            onConfirm = {},
            onCancel = {},
        )
    }
}

private fun GemPriceAlertSymbol.text(): String = when (this) {
    is GemPriceAlertSymbol.Currency -> java.util.Currency.getInstance(currency.toPrimitives().string).symbol
    GemPriceAlertSymbol.Percent -> "%"
}
