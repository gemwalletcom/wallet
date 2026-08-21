package com.gemwallet.android.features.swap.views

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.getBalanceInfo
import com.gemwallet.android.features.asset_select.presents.views.AssetSelectScreen
import com.gemwallet.android.features.asset_select.viewmodels.RecentsSheetViewModel
import com.gemwallet.android.features.swap.viewmodels.SwapSelectViewModel
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.AssetId

@Composable
fun SwapSelectScreen(
    onCancel: () -> Unit,
    onSelect: (select: SwapItemType, payId: AssetId?, receiveId: AssetId?) -> Unit,
    viewModel: SwapSelectViewModel = hiltViewModel(),
    recentsViewModel: RecentsSheetViewModel = hiltViewModel(),
) {
    val select by viewModel.select.collectAsStateWithLifecycle()
    val payId by viewModel.payAssetId.collectAsStateWithLifecycle()
    val receiveId by viewModel.receiveAssetId.collectAsStateWithLifecycle()

    val onSelectAsset: (AssetId) -> Unit = { assetId ->
        when (select) {
            SwapItemType.Pay -> onSelect(SwapItemType.Pay, assetId, receiveId)
            SwapItemType.Receive -> onSelect(SwapItemType.Receive, payId, assetId)
        }
    }

    AssetSelectScreen(
        title = when (select) {
            SwapItemType.Pay -> stringResource(id = R.string.swap_you_pay)
            SwapItemType.Receive -> stringResource(id = R.string.swap_you_receive)
        },
        titleBadge = { null },
        recentType = RecentType.SwapSelect,
        onCancel = onCancel,
        onSelect = onSelectAsset,
        onSelectRecent = onSelectAsset,
        itemTrailing = { getBalanceInfo(it)() },
        viewModel = viewModel,
        recentsViewModel = recentsViewModel,
    )
}
