package com.gemwallet.android.features.perpetual.views

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.perpetual.viewmodels.PerpetualsPreviewViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.models.ListPosition
import com.wallet.core.primitives.AssetId

@Composable
fun PerpetualsPreviewSection(
    onOpenPerpetuals: () -> Unit,
    onOpenPerpetualDetails: (AssetId) -> Unit,
    viewModel: PerpetualsPreviewViewModel = hiltViewModel(),
) {
    val show by viewModel.showPerpetuals.collectAsStateWithLifecycle()
    if (!show) return
    val positions by viewModel.positions.collectAsStateWithLifecycle()

    Column {
        SubheaderItem(stringResource(R.string.perpetuals_title), onClick = onOpenPerpetuals)
        if (positions.isEmpty()) {
            ListItem(
                model = viewModel.bannerListItem,
                listPosition = ListPosition.Single,
                modifier = Modifier.clickable(onClick = onOpenPerpetuals),
                accessory = { DataBadgeChevron() },
            )
        } else {
            positions.forEachIndexed { index, position ->
                ListItem(
                    model = position.model,
                    listPosition = ListPosition.getPosition(index, positions.size),
                    modifier = Modifier.clickable { onOpenPerpetualDetails(position.asset.id) },
                )
            }
        }
    }
}
