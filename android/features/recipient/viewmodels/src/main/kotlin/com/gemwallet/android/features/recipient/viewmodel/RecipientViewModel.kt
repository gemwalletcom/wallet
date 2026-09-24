package com.gemwallet.android.features.recipient.viewmodel

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.contacts.cases.GetContacts
import com.gemwallet.android.application.contacts.values.ContactRecipient
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.isMemoSupport
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.recipient.viewmodel.models.QrScanField
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientRowUIModel
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientState
import com.gemwallet.android.features.recipient.viewmodel.models.uiSection
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.components.fields.NameResolveIndicatorUIModel
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.models.name.AddressInputModel
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.optionalNft
import com.gemwallet.android.ui.models.navigation.optionalPaymentRecipient
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.gemwallet.android.ui.style.indicator
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.flow.updateAndGet
import kotlinx.coroutines.launch
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemRecipientException
import uniffi.gemstone.GemRecipientNext
import uniffi.gemstone.GemRecipientScan
import uniffi.gemstone.GemRecipientServiceInterface
import uniffi.gemstone.GemRecipientSession
import uniffi.gemstone.GemRecipientType
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class RecipientViewModel @Inject constructor(
    private val getSession: GetSession,
    private val getWallets: GetWallets,
    private val getContacts: GetContacts,
    private val getAssetInfo: GetAssetInfo,
    savedStateHandle: SavedStateHandle,
    private val service: GemRecipientServiceInterface,
    nameService: GemNameServiceInterface,
    @param:ApplicationContext private val context: Context,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val addressInput = AddressInputModel(nameService, viewModelScope)

    val address: StateFlow<String> = addressInput.text
    val nameResolveIndicator: StateFlow<NameResolveIndicatorUIModel?> = addressInput.nameResolveState.map { it.indicator() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
    val addressError: StateFlow<String> = addressInput.error
        .map { it?.string(context).orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val recipientInput = MutableStateFlow(GemRecipientSession(address = "", memo = "", payment = null))
    val memo: StateFlow<String> = recipientInput.map { it.memo }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assetId = savedStateHandle.requireAssetId(RouteArgument.AssetId)
    private val nft = savedStateHandle.optionalNft()

    val state: StateFlow<RecipientState> = getAssetInfo(assetId)
        .filterNotNull()
        .map { assetInfo ->
            val type = nft?.let { GemRecipientType.Nft(it.toGem()) } ?: GemRecipientType.Asset(assetInfo.asset.toGem())
            RecipientState.Ready(assetInfo.asset, type)
        }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, RecipientState.Loading)

    private val wallets = combine(session, getWallets()) { _, wallets -> wallets.map { it.toGem() } }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val contacts: StateFlow<List<ContactRecipient>> = state
        .flatMapLatest { state ->
            when (state) {
                RecipientState.Loading -> flowOf(emptyList())
                is RecipientState.Ready -> getContacts.getContactRecipients(state.asset.chain)
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val sections: StateFlow<List<ListSection<RecipientRowUIModel>>> = combine(wallets, contacts, state) { wallets, contacts, state ->
        when (state) {
            RecipientState.Loading -> emptyList()

            is RecipientState.Ready -> service.recipientSections(wallets, state.asset.chain.string, contacts.map { GemRecipient(address = it.address, name = it.name, memo = it.memo) })
                .mapIndexed { index, section -> section.uiSection(index.toString(), context) }
        }
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val buttonState: StateFlow<ButtonState> = addressInput.isValid
        .map { buttonState(enabled = it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

    init {
        viewModelScope.launch {
            state.filterIsInstance<RecipientState.Ready>()
                .collect { addressInput.setChain(it.asset.chain) }
        }
    }

    val hasMemo: StateFlow<Boolean> = state
        .map {
            when (it) {
                RecipientState.Loading -> false
                is RecipientState.Ready -> it.asset.chain.isMemoSupport()
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    init {
        savedStateHandle.optionalPaymentRecipient(RouteArgument.Payment)?.let(::updateFrom)
    }

    fun onValidateAddress() {
        addressInput.validate()
    }

    fun onNext(recipient: RecipientState.Ready, amountAction: AmountTransactionAction, confirmAction: ConfirmTransactionAction) {
        if (!addressInput.validate()) return
        val next = try {
            recipientInput.updateAndGet { it.onAddressChanged(address.value) }.next(recipient.type, addressInput.nameRecordState)
        } catch (rejection: GemRecipientException) {
            addressInput.markInvalid(rejection)
            return
        }
        route(recipient, next, amountAction, confirmAction)
    }

    fun onDestination(recipient: RecipientState.Ready, destination: GemRecipient, amountAction: AmountTransactionAction, confirmAction: ConfirmTransactionAction) {
        val next = try {
            service.select(recipient.type, destination)
        } catch (rejection: GemRecipientException) {
            addressInput.markInvalid(rejection)
            return
        }
        route(recipient, next, amountAction, confirmAction)
    }

    private fun route(recipient: RecipientState.Ready, next: GemRecipientNext, amountAction: AmountTransactionAction, confirmAction: ConfirmTransactionAction) {
        when (next) {
            is GemRecipientNext.Amount -> amountAction(
                AmountParams.Transfer(recipient.asset.id, next.payment),
            )

            is GemRecipientNext.Confirm -> confirmAction(ConfirmTransferInput(next.transfer))
        }
    }

    fun onAddress(input: String) {
        recipientInput.update { it.onAddressChanged(input) }
        addressInput.onTextChange(input)
    }

    fun onMemo(input: String) {
        recipientInput.update { it.onMemoChanged(input) }
    }

    fun setQrData(state: RecipientState.Ready, field: QrScanField, data: String, confirmAction: ConfirmTransactionAction) {
        when (field) {
            QrScanField.None -> Unit
            QrScanField.Memo -> onMemo(data)
            QrScanField.Address -> onAddressScan(state.type, data, confirmAction)
        }
    }

    private fun onAddressScan(type: GemRecipientType, data: String, confirmAction: ConfirmTransactionAction) {
        val scan = try {
            service.scan(data, type)
        } catch (rejection: GemRecipientException) {
            addressInput.markInvalid(rejection)
            return
        }
        when (scan) {
            is GemRecipientScan.Confirm -> confirmAction(ConfirmTransferInput(scan.transfer))
            is GemRecipientScan.Recipient -> updateFrom(scan.payment)
        }
    }

    private fun updateFrom(payment: GemPaymentRecipient) {
        recipientInput.update { it.onPayment(payment) }
        addressInput.setScannedAddress(payment.recipient.address)
    }
}
