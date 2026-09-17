package com.gemwallet.android.features.perpetual.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositions
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.config.showPerpetuals
import com.gemwallet.android.features.perpetual.viewmodels.models.PerpetualPositionRowUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.perpetual.listItem
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

@HiltViewModel
class PerpetualsPreviewViewModel @Inject constructor(
    userConfig: UserConfig,
    getSession: GetSession,
    getPositions: GetPerpetualPositions,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val bannerListItem = ListItemModel(
        title = context.getString(R.string.banner_perpetuals_title),
        image = ListItemImage.Drawable(R.drawable.settings_pricealert),
    )

    val showPerpetuals = userConfig.showPerpetuals(getSession())
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val positions = getPositions.getPerpetualPositions()
        .map { positions -> positions.map { PerpetualPositionRowUIModel(it.asset, it.listItem(context)) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
}
