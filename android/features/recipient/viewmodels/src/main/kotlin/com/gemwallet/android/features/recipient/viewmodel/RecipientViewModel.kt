package com.gemwallet.android.features.recipient.viewmodel

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.contacts.cases.GetContacts
import com.gemwallet.android.application.contacts.values.ContactRecipient
import com.gemwallet.android.application.nft.cases.GetAssetNft
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
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.ListSection
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.models.name.AddressInputModel
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.optionalNftAssetId
import com.gemwallet.android.ui.models.navigation.optionalPaymentRecipient
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.gemwallet.android.ui.style.indicator
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.NFTAsset
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAddressService
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemRecipientException
import uniffi.gemstone.GemRecipientNext
import uniffi.gemstone.GemRecipientScan
import uniffi.gemstone.GemRecipientServiceInterface
import uniffi.gemstone.GemRecipientType

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class RecipientViewModel @Inject constructor(
    private val getSession: GetSession,
    private val getWallets: GetWallets,
    private val getContacts: GetContacts,
    private val getAssetInfo: GetAssetInfo,
    private val getAssetNft: GetAssetNft,
    savedStateHandle: SavedStateHandle,
    private val service: GemRecipientServiceInterface,
    nameService: GemNameServiceInterface,
    private val addressService: GemAddressService,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val addressInput = AddressInputModel(nameService, viewModelScope)

    val address: StateFlow<String> = addressInput.text
    val nameResolveIndicator: StateFlow<NameResolveIndicatorUIModel?> = addressInput.nameResolveState.map { it.indicator() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
    val addressError: StateFlow<Boolean> = addressInput.showError

    private val _memo = MutableStateFlow("")
    val memo = _memo.asStateFlow()
    private var references = emptyList<String>()
    private var requestedAmount: String? = null

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assetId = savedStateHandle.requireAssetId(RouteArgument.AssetId)
    private val nftAssetId = savedStateHandle.optionalNftAssetId(RouteArgument.NftAssetId)

    private val nftAsset: Deferred<NFTAsset?> = viewModelScope.async(Dispatchers.IO, CoroutineStart.LAZY) {
        val id = nftAssetId ?: return@async null
        runCatching {
            getAssetNft.getAssetNft(id).first().assets.firstOrNull()
        }.getOrNull()
    }

    val state: StateFlow<RecipientState> = getAssetInfo(assetId)
        .filterNotNull()
        .map { assetInfo ->
            val type: GemRecipientType? = if (nftAssetId == null) {
                GemRecipientType.Asset(assetInfo.asset.toGem())
            } else {
                nftAsset.await()?.let { GemRecipientType.Nft(it.toGem()) }
            }
            type?.let { RecipientState.Ready(assetInfo.asset, it) } ?: RecipientState.Loading
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, RecipientState.Loading)

    private val wallets = combine(session, getWallets()) { _, wallets -> wallets.map { it.toGem() } }
        .flowOn(Dispatchers.IO)
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
            is RecipientState.Ready -> service.recipientSections(wallets, state.asset.chain.string, contacts.isNotEmpty())
                .mapIndexed { index, section -> section.uiSection(index.toString(), context, addressService, contacts, state.asset.chain) }
        }
    }
        .flowOn(Dispatchers.IO)
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

    fun onNext(
        recipient: RecipientState.Ready,
        amountAction: AmountTransactionAction,
        confirmAction: ConfirmTransactionAction,
    ) {
        if (!addressInput.validate()) return
        submit(recipient, address.value, addressInput.nameRecordState, amountAction, confirmAction)
    }

    fun onDestination(
        recipient: RecipientState.Ready,
        destination: GemRecipient,
        amountAction: AmountTransactionAction,
        confirmAction: ConfirmTransactionAction,
    ) {
        submit(recipient, destination.address, GemNameRecordState.None, amountAction, confirmAction, destination.name)
    }

    private fun submit(
        recipient: RecipientState.Ready,
        input: String,
        state: GemNameRecordState,
        amountAction: AmountTransactionAction,
        confirmAction: ConfirmTransactionAction,
        selectedName: String? = null,
    ) {
        val asset = recipient.asset
        val resolved = try {
            service.recipient(asset.chain.string, input, state, memo.value, references)
        } catch (_: GemRecipientException) {
            addressInput.markInvalid()
            return
        }
        val destination = GemRecipient(address = resolved.address, name = resolved.name ?: selectedName)
        when (val next = service.next(recipient.type, GemPaymentRecipient(destination, requestedAmount))) {
            is GemRecipientNext.Amount -> amountAction(
                AmountParams.Transfer(asset.id, next.payment.recipient, memo.value, references, next.payment.amount)
            )
            is GemRecipientNext.Confirm -> confirmAction(ConfirmTransferInput(next.transfer))
        }
    }

    fun onAddress(input: String) {
        if (input != address.value) {
            requestedAmount = null
            references = emptyList()
        }
        addressInput.onTextChange(input)
    }

    fun onMemo(input: String) {
        _memo.value = input
    }

    fun setQrData(state: RecipientState.Ready, field: QrScanField, data: String, confirmAction: ConfirmTransactionAction) {
        when (field) {
            QrScanField.None -> Unit
            QrScanField.Memo -> _memo.value = data
            QrScanField.Address -> onAddressScan(state.type, data, confirmAction)
        }
    }

    private fun onAddressScan(type: GemRecipientType, data: String, confirmAction: ConfirmTransactionAction) {
        val scan = try {
            service.scan(data, type)
        } catch (_: GemRecipientException) {
            addressInput.markInvalid()
            return
        }
        when (scan) {
            is GemRecipientScan.Confirm -> confirmAction(ConfirmTransferInput(scan.transfer))
            is GemRecipientScan.Recipient -> updateFrom(scan.payment)
        }
    }

    private fun updateFrom(payment: GemPaymentRecipient) {
        addressInput.setScannedAddress(payment.recipient.address)
        payment.recipient.memo?.let { _memo.value = it }
        references = payment.recipient.references
        requestedAmount = payment.amount
    }

}
