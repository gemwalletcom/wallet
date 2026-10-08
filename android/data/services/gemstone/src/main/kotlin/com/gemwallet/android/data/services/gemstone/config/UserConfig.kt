package com.gemwallet.android.data.services.gemstone.config

import com.gemwallet.android.application.preferences.cases.ObservablePreferences
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.Appearance
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import uniffi.gemstone.GemPreferencesObserver
import uniffi.gemstone.GemPreferencesServiceInterface

class UserConfig(private val preferencesService: GemPreferencesServiceInterface) : ObservablePreferences {

    override fun developEnabled(): Boolean = preferencesService.isDeveloperEnabled()

    override fun developEnabled(enabled: Boolean) = preferencesService.setDeveloperEnabled(enabled)

    fun increaseLaunchNumber() {
        preferencesService.incrementLaunchesCount()
    }

    fun shouldRequestReview(): Boolean = preferencesService.shouldRequestReview()

    fun setRateApplicationShown() = preferencesService.setRateApplicationShown()

    private val hideBalancesState = MutableStateFlow(preferencesService.isHideBalanceEnabled())
    private val perpetualEnabledState = MutableStateFlow(preferencesService.isPerpetualEnabled())
    private val appearanceState = MutableStateFlow(preferencesService.getAppearance().toPrimitives())
    private val termsAcceptedState = MutableStateFlow(preferencesService.isAcceptTermsCompleted())

    override fun isHideBalances(): Flow<Boolean> = hideBalancesState

    override fun hideBalances() {
        preferencesService.setHideBalanceEnabled(!preferencesService.isHideBalanceEnabled())
        hideBalancesState.value = preferencesService.isHideBalanceEnabled()
    }

    override fun isPerpetualEnabled(): Flow<Boolean> = perpetualEnabledState

    override fun setPerpetualEnabled(enabled: Boolean) {
        preferencesService.setPerpetualEnabled(enabled)
        perpetualEnabledState.value = preferencesService.isPerpetualEnabled()
    }

    override fun appearance(): Flow<Appearance> = appearanceState

    override fun setAppearance(appearance: Appearance) {
        preferencesService.setAppearance(appearance.toGem())
        appearanceState.value = preferencesService.getAppearance().toPrimitives()
    }

    init {
        preferencesService.setObserver(
            object : GemPreferencesObserver {
                override fun onPreferencesChanged() = reload()
            },
        )
    }

    private fun reload() {
        hideBalancesState.value = preferencesService.isHideBalanceEnabled()
        perpetualEnabledState.value = preferencesService.isPerpetualEnabled()
        appearanceState.value = preferencesService.getAppearance().toPrimitives()
        termsAcceptedState.value = preferencesService.isAcceptTermsCompleted()
    }

    fun isTermsAccepted(): Flow<Boolean> = termsAcceptedState

    fun acceptTerms() {
        preferencesService.setAcceptTermsCompleted()
        termsAcceptedState.value = preferencesService.isAcceptTermsCompleted()
    }

    fun shouldOfferAuthentication(isAvailable: Boolean, authRequired: Boolean): Boolean = preferencesService.shouldOfferAuthentication(isAvailable, authRequired)

    fun setAuthenticationOffered() {
        preferencesService.setAuthenticationOffered()
    }
}
