package com.gemwallet.android.features.transfer.viewmodels.amount

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.data.services.store.queries.DelegationQuery
import com.gemwallet.android.data.services.store.queries.PerpetualQuery
import com.gemwallet.android.data.services.store.queries.ValidatorQuery
import com.gemwallet.android.data.services.store.queries.ValidatorsQuery
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.transfer.viewmodels.amount.models.AmountExtrasUIModel
import com.gemwallet.android.features.transfer.viewmodels.amount.models.AmountUIState
import com.gemwallet.android.features.transfer.viewmodels.amount.models.ValidatorSelectUIModel
import com.gemwallet.android.features.transfer.viewmodels.amount.providers.AmountPerpetualProvider
import com.gemwallet.android.features.transfer.viewmodels.amount.providers.AmountStakeProvider
import com.gemwallet.android.math.numberFormat
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.fields.AmountSymbolUIModel
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.style.amountSymbol
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetData
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.StakeProviderType
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.EarnType
import uniffi.gemstone.GemAmountEntry
import uniffi.gemstone.GemAmountErrorDisplay
import uniffi.gemstone.GemAmountException
import uniffi.gemstone.GemAmountInput
import uniffi.gemstone.GemAmountInputType
import uniffi.gemstone.GemAmountRequest
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAmountSession
import uniffi.gemstone.GemAmountTitle
import uniffi.gemstone.GemAmountTransfer
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.newAmountSession
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AmountViewModel @Inject constructor(
    private val service: GemAmountServiceInterface,
    stakeService: GemStakeServiceInterface,
    getCurrentWalletId: GetCurrentWalletId,
    assetQuery: AssetQuery,
    perpetualQuery: PerpetualQuery,
    delegationQuery: DelegationQuery,
    validatorQuery: ValidatorQuery,
    validatorsQuery: ValidatorsQuery,
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
    @param:ApplicationContext private val context: Context,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val params: AmountParams = savedStateHandle.requireAmountParams()

    private val stakeProvider = (params as? AmountParams.Stake)?.let { AmountStakeProvider(it, validatorsQuery(it.assetId, StakeProviderType.Stake), stakeService, viewModelScope, ioDispatcher) }

    val perpetualProvider = (params as? AmountParams.Perpetual)?.let { AmountPerpetualProvider(it, context, service, getCurrentWalletId, assetQuery, perpetualQuery, viewModelScope) }

    private val assetInfo: StateFlow<AssetData?> = perpetualProvider?.assetInfo
        ?: getCurrentWalletId().flatMapLatest { walletId -> assetQuery(walletId.id, params.assetId) }
            .flowOn(ioDispatcher)
            .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val request: StateFlow<GemAmountRequest?> = stakeProvider?.request ?: perpetualProvider?.request ?: when (params) {
        is AmountParams.Transfer -> flowOf(GemAmountRequest.Transfer(GemAmountTransfer.Send(params.payment)))

        is AmountParams.Deposit -> flowOf(GemAmountRequest.Transfer(GemAmountTransfer.Deposit))

        is AmountParams.Withdraw -> flowOf(GemAmountRequest.Transfer(GemAmountTransfer.Withdraw))

        is AmountParams.Earn.Deposit -> assetInfo.map { current -> current?.let { validatorQuery(it.asset.id, params.providerId)?.toGem() }?.let { GemAmountRequest.Earn(EarnType.Deposit(it)) } }

        is AmountParams.Earn.Withdraw -> getSession()
            .flatMapLatest { session -> session?.wallet?.id?.let { delegationQuery(it, params.validatorId, params.delegationId) } ?: flowOf(null) }
            .map { delegation -> delegation?.let { GemAmountRequest.Earn(EarnType.Withdraw(it.toGem())) } }

        is AmountParams.Stake, is AmountParams.Perpetual -> flowOf(null)
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val amountType: StateFlow<GemAmountType?> = request
        .map { it?.amountType() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val input: StateFlow<GemAmountInput?> = combine(request, assetInfo) { request, current ->
        if (request == null || current == null) null else request.input(current.toGem())
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val extras: StateFlow<AmountExtrasUIModel> = stakeProvider?.extras ?: perpetualProvider?.extras ?: amountType
        .map { type -> (type as? GemAmountType.Earn)?.let { AmountExtrasUIModel.EarnProvider(it.provider) } ?: AmountExtrasUIModel.None }
        .stateIn(viewModelScope, SharingStarted.Eagerly, AmountExtrasUIModel.None)

    private val session = MutableStateFlow(newAmountSession(numberFormat()))

    var amount by mutableStateOf("")
        private set

    val amountInputType: StateFlow<GemAmountInputType> = session.map { it.inputType }
        .stateIn(viewModelScope, SharingStarted.Eagerly, GemAmountInputType.ASSET)
    private val amountError = MutableStateFlow<Throwable?>(null)

    val currency: Currency = service.getCurrency().toPrimitives()

    private val amountSymbol: StateFlow<AmountSymbolUIModel> = combine(amountInputType, assetInfo) { inputType, current ->
        inputType.amountSymbol(current?.asset?.symbol.orEmpty(), currency)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, GemAmountInputType.ASSET.amountSymbol("", currency))

    private val entry: StateFlow<GemAmountEntry?> = combine(session, assetInfo, amountType, input) { session, current, amountType, input ->
        if (current == null || amountType == null || input == null) {
            null
        } else {
            session.entry(amountType, current.asset.toGem(), input, current.price?.price, currency.toGem())
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val availableBalanceFormatted: StateFlow<String> = input.map { input ->
        input?.balance?.text().orEmpty()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val reserveForFeeFormatted: StateFlow<String?> = entry.map { entry ->
        entry?.reservedFee?.text()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val amountEquivalent: StateFlow<String> = entry.map { entry ->
        entry?.equivalent?.text().orEmpty()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val buttonState: StateFlow<ButtonState> = entry.map { entry ->
        buttonState(enabled = entry?.allowsConfirm() == true)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

    private val amountAsset: StateFlow<Asset?> = combine(assetInfo, request) { current, request ->
        val asset = current?.asset ?: return@combine null
        request?.displayAsset(asset.toGem())?.toPrimitives() ?: asset
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val amountErrorDisplay: StateFlow<GemAmountErrorDisplay?> = amountError
        .map { (it as? GemAmountException)?.display() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val uiState: StateFlow<AmountUIState> = combine(
        combine(amountType, amountAsset, amountSymbol, input) { type, asset, symbol, input -> listOf(type?.title(), asset, symbol, input) },
        combine(availableBalanceFormatted, reserveForFeeFormatted, amountEquivalent, buttonState) { available, reserve, equivalent, button -> listOf(available, reserve, equivalent, button) },
        combine(amountType, amountErrorDisplay, extras) { amountType, errorDisplay, extras -> listOf(amountType, errorDisplay, extras) },
    ) { screen, values, rest ->
        val input = screen[3] as GemAmountInput?
        val errorDisplay = rest[1] as GemAmountErrorDisplay?
        AmountUIState(
            title = (screen[0] as GemAmountTitle?)?.text(context).orEmpty(),
            asset = screen[1] as Asset?,
            icon = input?.icon,
            amountSymbol = screen[2] as AmountSymbolUIModel,
            canSwitchInputType = (rest[0] as GemAmountType?)?.canSwitchInputType() == true,
            readOnly = input?.canChangeValue == false,
            focusesInput = input?.focusesInput == true,
            showsAssetBalance = input?.showsAssetBalance != false,
            usesWholeAmounts = input?.usesWholeAmounts == true,
            availableBalance = values[0] as String,
            reserveForFee = values[1] as String?,
            equivalent = values[2] as String,
            error = errorDisplay?.text(context).orEmpty(),
            errorTopic = errorDisplay?.info(),
            buttonState = values[3] as ButtonState,
            extras = rest[2] as AmountExtrasUIModel,
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, AmountUIState())

    val validatorPicker: StateFlow<ValidatorSelectUIModel?> = stakeProvider?.let { stake ->
        combine(stake.validatorOptions, stake.selectedValidatorId) { options, selectedId ->
            options?.let { ValidatorSelectUIModel(selection = it, selectedId = selectedId.orEmpty()) }
        }.stateIn(viewModelScope, SharingStarted.Eagerly, null)
    } ?: MutableStateFlow(null)

    fun selectValidator(id: String?) {
        stakeProvider?.selectValidator(id)
    }

    fun selectResource(resource: Resource) {
        stakeProvider?.setResource(resource)
    }

    fun selectLeverage(value: UByte) {
        perpetualProvider?.setLeverage(value)
    }

    init {
        entry
            .onEach { amountError.value = it?.error }
            .launchIn(viewModelScope)

        viewModelScope.launch { prefillAmount(input.filterNotNull().first()) }
    }

    private fun prefillAmount(input: GemAmountInput) {
        val current = assetInfo.value ?: return
        showSession(session.value.onPrefill(input, current.asset.toGem()))
    }

    fun updateAmount(input: String) {
        amount = input
        session.update { it.onText(input) }
    }

    fun onMaxAmount() {
        val current = assetInfo.value ?: return
        val input = input.value ?: return
        showSession(session.value.onMax(input, current.asset.toGem()))
    }

    fun switchInputType() {
        showSession(session.value.onToggle())
    }

    private fun showSession(next: GemAmountSession) {
        session.value = next
        amount = next.text
    }

    fun onNext(onConfirm: (ConfirmTransferInput) -> Unit) {
        viewModelScope.launch {
            val entry = entry.value ?: return@launch
            entry.error?.let {
                amountError.value = it
                return@launch
            }
            val value = entry.value ?: return@launch
            try {
                amountError.value = null
                val current = assetInfo.filterNotNull().first()
                val request = request.filterNotNull().first()
                onConfirm(ConfirmTransferInput(service.transferData(current.asset.toGem(), request, value, entry.isMax)))
            } catch (err: CancellationException) {
                throw err
            } catch (err: Throwable) {
                amountError.value = err
            }
        }
    }
}
