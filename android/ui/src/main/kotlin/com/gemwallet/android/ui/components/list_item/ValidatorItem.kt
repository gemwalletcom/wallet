package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.components.image.iconResource
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme
import uniffi.gemstone.DelegationValidator
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemValidatorRow
import uniffi.gemstone.StakeProviderType

@Composable
fun ValidatorItem(data: GemValidatorRow, listPosition: ListPosition, isSelected: Boolean = false, onClick: ((String) -> Unit)?) {
    ListItem(
        modifier = Modifier.clickable(enabled = onClick != null) { onClick?.invoke(data.validator.id) },
        leading = {
            ValidatorIcon(data = data, isSelected = isSelected)
        },
        title = {
            Text(
                text = data.name,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
        },
        listPosition = listPosition,
        trailing = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                ListItemSupportText(data.apr.string(LocalContext.current))
            }
        },
    )
}

val GemValidatorRow.icon: Any
    get() = provider?.iconResource() ?: imageUrl

@Composable
private fun ValidatorIcon(data: GemValidatorRow, isSelected: Boolean) {
    if (isSelected) {
        IconWithBadge(
            icon = data.icon,
            placeholder = data.placeholder,
            badge = { SelectionCheckmark() },
        )
    } else {
        IconWithBadge(
            icon = data.icon,
            placeholder = data.placeholder,
        )
    }
}

@Composable
@Preview
fun PreviewValidatorItem() {
    WalletTheme {
        ValidatorItem(
            data = previewValidatorRow(),
            isSelected = false,
            listPosition = ListPosition.Middle,
            onClick = {},
        )
    }
}

@Composable
@Preview
fun PreviewValidatorItemSelected() {
    WalletTheme {
        ValidatorItem(
            data = previewValidatorRow(),
            listPosition = ListPosition.Single,
            isSelected = true,
            onClick = {},
        )
    }
}

private fun previewValidatorRow(id: String = "some_validator_id", name: String = "Castlenode") = GemValidatorRow(
    validator = DelegationValidator(
        chain = "cosmos",
        id = id,
        name = name,
        isActive = true,
        commission = 0.0,
        apr = 9.1,
        providerType = StakeProviderType.STAKE,
    ),
    name = name,
    imageUrl = "",
    placeholder = name.take(1),
    provider = null,
    apr = GemLocalizedText.Apr(null),
    explorer = null,
)
