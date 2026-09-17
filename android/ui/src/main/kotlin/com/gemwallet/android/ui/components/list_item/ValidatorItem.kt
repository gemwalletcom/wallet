package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.domains.duration.formatAvailableIn
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.IconWithBadge
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme
import com.wallet.core.primitives.Delegation

@Composable
fun ValidatorItem(
    data: ValidatorRowUIModel,
    listPosition: ListPosition,
    isSelected: Boolean = false,
    onClick: ((String) -> Unit)?
) {
    ListItem(
        modifier = Modifier.clickable(enabled = onClick != null) { onClick?.invoke(data.id) },
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
                ListItemSupportText(R.string.stake_apr, " ${data.aprText}")
            }
        },
    )
}

@Composable
private fun ValidatorIcon(
    data: ValidatorRowUIModel,
    isSelected: Boolean,
) {
    if (isSelected) {
        IconWithBadge(
            icon = data.imageUrl,
            placeholder = data.placeholder,
            badge = { SelectionCheckmark() },
        )
    } else {
        IconWithBadge(
            icon = data.imageUrl,
            placeholder = data.placeholder,
        )
    }
}


fun availableIn(delegation: Delegation?): String {
    val remaining = availableInDurationMillis(delegation) ?: return ""
    return formatAvailableIn(remaining)
}

internal fun availableInDurationMillis(
    delegation: Delegation?,
    currentTimeMillis: Long = System.currentTimeMillis(),
): Long? = delegation?.base?.completionDate?.minus(currentTimeMillis)

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

private fun previewValidatorRow() = ValidatorRowUIModel(
    id = "some_validator_id",
    name = "Castlenode",
    imageUrl = "",
    placeholder = "C",
    aprText = 9.10.formatApr(),
)
