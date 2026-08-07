package com.gemwallet.android.data.coordinators.device

import com.gemwallet.android.application.device.coordinators.EnableDevicePush
import com.gemwallet.android.cases.device.SwitchPushEnabled
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import kotlinx.coroutines.flow.firstOrNull

class EnableDevicePushImpl(
    private val switchPushEnabled: SwitchPushEnabled,
    private val walletsRepository: WalletsRepository,
) : EnableDevicePush {

    override suspend fun invoke() {
        switchPushEnabled.switchPushEnabled(true, walletsRepository.getAll().firstOrNull() ?: emptyList())
    }
}
