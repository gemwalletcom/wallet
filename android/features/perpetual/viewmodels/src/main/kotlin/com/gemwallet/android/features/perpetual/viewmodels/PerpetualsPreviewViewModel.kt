package com.gemwallet.android.features.perpetual.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositions
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.domains.balance.hiddenWhen
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.perpetual.viewmodels.models.PerpetualPositionRowUIModel
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.perpetual.listItem
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.perpetualBalanceTotal
import javax.inject.Inject

@HiltViewModel
class PerpetualsPreviewViewModel @Inject constructor(userConfig: UserConfig, getPositions: GetPerpetualPositions, getBalance: GetPerpetualBalance, @param:ApplicationContext private val context: Context) : ViewModel() {

    val tradeListItem = combine(getBalance.getBalance(), userConfig.isHideBalances()) { balance, hideBalance ->
        ListItemModel(
            title = context.getString(R.string.perpetuals_trade),
            subtitle = perpetualBalanceTotal(balance?.toGem()).text().hiddenWhen(hideBalance),
        )
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ListItemModel(title = context.getString(R.string.perpetuals_trade)))

    val positions = combine(getPositions.getPerpetualPositions(), userConfig.isHideBalances()) { positions, hideBalance ->
        positions.map { PerpetualPositionRowUIModel(it.asset, it.listItem(context, hideBalance)) }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
}
