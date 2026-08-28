package com.gemwallet.android.data.repositories.config

import android.content.Context
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import com.gemwallet.android.data.service.store.ConfigStore
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Appearance
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.GemPreferencesService
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map

private val Context.dataStore by preferencesDataStore(name = "user_config")

class UserConfig(
    private val context: Context,
    private val configStore: ConfigStore,
    private val preferencesService: GemPreferencesService,
) {

    fun authRequired(): Boolean = configStore.getBoolean(ConfigKey.Auth.string)

    fun setAuthRequired(enabled: Boolean) = configStore.putBoolean(ConfigKey.Auth.string, enabled)

    fun developEnabled(): Boolean = preferencesService.isDeveloperEnabled()

    fun developEnabled(enabled: Boolean) = preferencesService.setDeveloperEnabled(enabled)

    fun increaseLaunchNumber() {
        preferencesService.incrementLaunchesCount()
    }

    fun shouldRequestReview(): Boolean = preferencesService.shouldRequestReview()

    fun setRateApplicationShown() = preferencesService.setRateApplicationShown()

    fun chartPeriod(): ChartPeriod = preferencesService.getChartPeriod().decodeJson()

    fun setChartPeriod(period: ChartPeriod) = preferencesService.setChartPeriod(period.toJson())

    fun perpetualChartPeriod(): ChartPeriod = preferencesService.getPerpetualChartPeriod().decodeJson()

    fun setPerpetualChartPeriod(period: ChartPeriod) = preferencesService.setPerpetualChartPeriod(period.toJson())

    private val hideBalancesState = MutableStateFlow(preferencesService.isHideBalanceEnabled())
    private val perpetualEnabledState = MutableStateFlow(preferencesService.isPerpetualEnabled())
    private val appearanceState = MutableStateFlow(preferencesService.getAppearance().decodeJson<Appearance>())
    private val termsAcceptedState = MutableStateFlow(preferencesService.isAcceptTermsCompleted())
    private val askNotificationsState = MutableStateFlow(preferencesService.shouldAskNotifications())

    fun isHideBalances(): Flow<Boolean> = hideBalancesState

    fun hideBalances() {
        preferencesService.setHideBalanceEnabled(!preferencesService.isHideBalanceEnabled())
        hideBalancesState.value = preferencesService.isHideBalanceEnabled()
    }

    fun isPerpetualEnabled(): Flow<Boolean> = perpetualEnabledState

    fun setPerpetualEnabled(enabled: Boolean) {
        preferencesService.setPerpetualEnabled(enabled)
        perpetualEnabledState.value = preferencesService.isPerpetualEnabled()
    }

    fun appearance(): Flow<Appearance> = appearanceState

    fun setAppearance(appearance: Appearance) {
        preferencesService.setAppearance(appearance.toJson())
        appearanceState.value = preferencesService.getAppearance().decodeJson()
    }

    private val perpetualLeverageState = MutableStateFlow(preferencesService.getPerpetualLeverage().toInt())
    private val perpetualTakeProfitState = MutableStateFlow(preferencesService.getPerpetualTakeProfitPercent().toInt())
    private val perpetualStopLossState = MutableStateFlow(preferencesService.getPerpetualStopLossPercent().toInt())
    private val swapSlippageBpsState = MutableStateFlow(preferencesService.getSwapSlippageBps())

    fun perpetualLeverage(): Flow<Int> = perpetualLeverageState

    fun setPerpetualLeverage(value: Int) {
        preferencesService.setPerpetualLeverage(value.toUByte())
        perpetualLeverageState.value = preferencesService.getPerpetualLeverage().toInt()
    }

    fun perpetualTakeProfit(): Flow<Int> = perpetualTakeProfitState

    fun setPerpetualTakeProfit(value: Int) {
        preferencesService.setPerpetualTakeProfitPercent(value.toUByte())
        perpetualTakeProfitState.value = preferencesService.getPerpetualTakeProfitPercent().toInt()
    }

    fun perpetualStopLoss(): Flow<Int> = perpetualStopLossState

    fun setPerpetualStopLoss(value: Int) {
        preferencesService.setPerpetualStopLossPercent(value.toUByte())
        perpetualStopLossState.value = preferencesService.getPerpetualStopLossPercent().toInt()
    }

    fun swapSlippageBps(): Flow<UInt?> = swapSlippageBpsState

    fun setSwapSlippageBps(bps: UInt?) {
        preferencesService.setSwapSlippageBps(bps)
        swapSlippageBpsState.value = preferencesService.getSwapSlippageBps()
    }

    fun reload() {
        hideBalancesState.value = preferencesService.isHideBalanceEnabled()
        perpetualEnabledState.value = preferencesService.isPerpetualEnabled()
        appearanceState.value = preferencesService.getAppearance().decodeJson()
        termsAcceptedState.value = preferencesService.isAcceptTermsCompleted()
        askNotificationsState.value = preferencesService.shouldAskNotifications()
        perpetualLeverageState.value = preferencesService.getPerpetualLeverage().toInt()
        perpetualTakeProfitState.value = preferencesService.getPerpetualTakeProfitPercent().toInt()
        perpetualStopLossState.value = preferencesService.getPerpetualStopLossPercent().toInt()
        swapSlippageBpsState.value = preferencesService.getSwapSlippageBps()
    }

    fun getLockInterval(): Flow<Int> = read(Key.LockInterval, 1)

    suspend fun setLockInterval(minutes: Int) = write(Key.LockInterval, minutes)

    fun isTermsAccepted(): Flow<Boolean> = termsAcceptedState

    fun acceptTerms() {
        preferencesService.setAcceptTermsCompleted()
        termsAcceptedState.value = preferencesService.isAcceptTermsCompleted()
    }

    fun isAskNotifications(): Flow<Boolean> = askNotificationsState

    fun stopAskNotifications() {
        preferencesService.setNotificationsAsked()
        askNotificationsState.value = preferencesService.shouldAskNotifications()
    }

    private fun <T> read(key: Preferences.Key<T>, default: T): Flow<T> =
        context.dataStore.data.map { it[key] ?: default }

    private suspend fun <T> write(key: Preferences.Key<T>, value: T) {
        context.dataStore.edit { it[key] = value }
    }

    private enum class ConfigKey(val string: String) {
        Auth("auth"),
        ;
    }

    private object Key {
        val LockInterval = intPreferencesKey("lock_interval")
    }
}
