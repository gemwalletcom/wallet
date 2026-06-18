package com.gemwallet.android.features.confirm.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.confirm.coordinators.BuildConfirmProperties
import com.gemwallet.android.application.confirm.coordinators.ConfirmTransaction
import com.gemwallet.android.application.confirm.coordinators.ValidateBalance
import com.gemwallet.android.cases.addresses.GetAddressName
import com.gemwallet.android.blockchain.services.SignerPreloaderProxy
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.transactions.TransactionBalanceService
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.perpetual.PerpetualConfirmDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelInput
import com.gemwallet.android.ui.models.swap.SwapProviderUIModelFactory
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.domains.confirm.AmountUIModel
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.features.confirm.models.PerpetualModifyAutocloseFactory
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualType
import com.wallet.core.primitives.FeePriority
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import com.wallet.core.primitives.SimulationResult
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ConfirmViewModel @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
    private val signerPreload: SignerPreloaderProxy,
    private val transactionBalanceService: TransactionBalanceService,
    private val validateBalance: ValidateBalance,
    private val confirmTransaction: ConfirmTransaction,
    private val buildConfirmProperties: BuildConfirmProperties,
    private val getCurrentBlockExplorer: GetCurrentBlockExplorer,
    private val getAddressName: GetAddressName,
    private val savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val restart = MutableStateFlow(false)
    val state = MutableStateFlow<ConfirmState>(ConfirmState.Prepare)
    val feePriority = MutableStateFlow(FeePriority.Normal)
    private val walletConnectSimulationState = MutableStateFlow<SimulationResult?>(null)

    private val request = savedStateHandle.getStateFlow<String?>(RouteArgument.Params.key, null)
        .combine(restart) { request, _ -> request }
        .filterNotNull()
        .mapNotNull { paramsPack ->
            state.update { ConfirmState.Prepare }
            ConfirmParams.unpack(paramsPack)
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val session = sessionRepository.session()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val walletConnectHeaderAssetInfo = walletConnectSimulationState
        .map { it?.header?.assetId }
        .distinctUntilChanged()
        .flatMapLatest { assetId ->
            if (assetId == null) return@flatMapLatest flowOf(null)
            assetsRepository.getTokenInfo(assetId).also {
                if (assetId.tokenId != null && it.firstOrNull() == null) {
                    assetsRepository.searchToken(assetId, sessionRepository.getCurrentCurrency())
                }
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val walletConnectReview = combine(walletConnectSimulationState, walletConnectHeaderAssetInfo, request) { simulation, headerAssetInfo, params ->
        val chain = params?.assetId?.chain
        val explorerName = chain?.let { getCurrentBlockExplorer.getCurrentBlockExplorer(it) }
        val review = simulation?.toWalletConnectReview(chain, explorerName) ?: WalletConnectReview()
        val headerAssetId = simulation?.header?.assetId
        val asset = when {
            headerAssetId == null || params == null -> null
            headerAssetId == params.assetId -> params.asset
            else -> headerAssetInfo?.asset
        }
        review.copy(headerAsset = asset)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, WalletConnectReview())

    private val assetsInfo = request.filterNotNull().mapNotNull {
        if (it is ConfirmParams.SwapParams) {
            listOf(it.fromAsset.id, it.toAsset.id)
        } else {
            listOf(it.assetId)
        }
    }
    .flatMapLatest { assetsRepository.getAssetsInfo(it) }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val preloadData = combine(
        session,
        request.filterNotNull(),
        feePriority,
    ) { session, request, feePriority ->
        val owner = session?.wallet?.getAccount(request.assetId.chain)
        if (owner == null) {
            state.update { ConfirmState.FatalError("Session not found") }
            return@combine null
        }

        val preload = try {
            signerPreload.preload(params = request, feePriority = feePriority)
        } catch (err: Throwable) {
            state.update {
                ConfirmState.Error(err.toPreloadConfirmError(owner.chain))
            }
            return@combine null
        }

        val finalAmount = when {
            preload.input is ConfirmParams.Stake.RewardsParams -> preload.input.amount
            preload.input.useMaxAmount && preload.input.assetId == preload.fee().feeAssetId ->
                preload.input.amount - preload.fee().amount
            else -> preload.input.amount
        }
        state.update { ConfirmState.Ready }

        preload.copy(finalAmount = finalAmount)
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeAssetInfo = preloadData.flatMapLatest { signerParams ->
        if (signerParams == null) {
            flowOf(null)
        } else {
            assetsRepository.getAssetInfo(signerParams.fee().feeAssetId)
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val amountUIModel = combine(request, assetsInfo, preloadData) { request, assetsInfo, signerParams ->
        val fromAssetId = request?.assetId ?: return@combine null
        val assetInfo = assetsInfo?.getByAssetId(fromAssetId) ?: return@combine null
        val toAssetInfo = if (request is ConfirmParams.SwapParams) {
            assetsInfo.getByAssetId(request.toAsset.id) ?: return@combine null
        } else {
            null
        }

        val amount = Crypto(signerParams?.finalAmount ?: request.amount)

        AmountUIModel(
            transactionType = request.getTransactionType(),
            amount = amount.atomicValue,
            asset = assetInfo,
            fromAsset = assetInfo,
            fromAmount = amount.atomicValue.toString(),
            toAsset = toAssetInfo,
            toAmount = (request as? ConfirmParams.SwapParams)?.toAmount?.toString(),
            nftAsset = (request as? ConfirmParams.NftParams)?.nftAsset,
            price = assetInfo.price?.price?.price,
            currency = assetInfo.price?.currency ?: Currency.USD,
        )
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val perpetualType = request
        .map { (it as? ConfirmParams.PerpetualParams)?.perpetualType }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val detailElements = combine(request, assetsInfo, ::buildDetailElements)
        .distinctUntilChanged()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val recipientAddressName = request
        .filterNotNull()
        .map { it.assetId.chain to it.destination()?.address?.takeIf { address -> address.isNotEmpty() } }
        .distinctUntilChanged()
        .flatMapLatest { (chain, address) ->
            if (address == null) flowOf(null) else getAddressName.getAddressNameFlow(chain, address)
        }

    val txProperties = combine(request, assetsInfo, recipientAddressName) { request, assetsInfo, addressName ->
        request ?: return@combine emptyList()
        assetsInfo ?: return@combine emptyList()
        buildConfirmProperties(request, assetsInfo, addressName)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val feeUIModel = combine(preloadData, feeAssetInfo, state) { signerParams, feeAssetInfo, state ->
        val amount = signerParams?.fee()?.amount
        val result = if (state is ConfirmState.Prepare) {
            FeeUIModel.Calculating
        } else if (amount == null || feeAssetInfo == null) {
            if (state is ConfirmState.Error) FeeUIModel.Error else FeeUIModel.Calculating
        } else {
            try {
                val sendAssetInfo = assetsInfo.value?.getByAssetId(signerParams.input.assetId)
                if (sendAssetInfo != null) {
                    validateBalance(
                        signerParams,
                        sendAssetInfo,
                        feeAssetInfo,
                        getBalance(sendAssetInfo, signerParams.input)
                    )
                }
            } catch (err: ConfirmError) {
                this@ConfirmViewModel.state.update { ConfirmState.Error(err) }
            }

            FeeUIModel.FeeInfo(
                amount = amount,
                feeAsset = feeAssetInfo.asset,
                price = feeAssetInfo.price?.price?.price,
                currency = feeAssetInfo.price?.currency ?: Currency.USD,
                priority = signerParams.fee().priority,
            )
        }
        result
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeValue = feeUIModel.map { (it as? FeeUIModel.FeeInfo)?.cryptoAmountWithFiat.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val feeRates = preloadData.map { it?.feeRates.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun init(params: ConfirmParams, walletConnectSimulation: SimulationResult? = null) {
        viewModelScope.launch(Dispatchers.IO) {
            walletConnectSimulationState.value = walletConnectSimulation
            val pack = params.pack()
            if (savedStateHandle.get<String?>(RouteArgument.Params.key) == pack) {
                return@launch
            }
            state.update { ConfirmState.Prepare }
            savedStateHandle[RouteArgument.Params.key] = pack
        }
    }

    fun changeFeePriority(feePriority: FeePriority) {
        val selectedPriority = preloadData.value?.fee()?.priority ?: this.feePriority.value
        if (feePriority == selectedPriority) {
            return
        }
        state.update { ConfirmState.Prepare }
        this.feePriority.update { feePriority }
    }

    fun send(finishAction: FinishConfirmAction) = viewModelScope.launch(Dispatchers.IO) {
        if (state.value is ConfirmState.Error) {
            restart.update { !it }
            return@launch
        }
        state.update { ConfirmState.Sending }

        val signerParams = preloadData.value
        val assetInfo = assetsInfo.value?.getByAssetId(signerParams?.input?.assetId ?: return@launch)
        val feeAssetInfo = feeAssetInfo.value
        val session = session.value

        try {
            if (assetInfo == null || assetInfo.owner == null || session == null || feeAssetInfo == null) {
                throw ConfirmError.TransactionIncorrect
            }
            validateBalance(
                signerParams,
                assetInfo,
                feeAssetInfo,
                getBalance(assetInfo, signerParams.input),
            )
            val transactionHash = confirmTransaction(signerParams, session, assetInfo, viewModelScope)
            state.update { ConfirmState.Result(transactionHash = transactionHash) }
            viewModelScope.launch(Dispatchers.Main) {
                finishAction(transactionHash)
            }
        } catch (err: Throwable) {
            state.update { ConfirmState.BroadcastError(err.toBroadcastConfirmError()) }
        }
    }

    private suspend fun getBalance(assetInfo: AssetInfo, params: ConfirmParams): BigInteger {
        return transactionBalanceService.getBalance(assetInfo, params)
    }

    private fun List<AssetInfo>.getByAssetId(assetId: AssetId): AssetInfo? {
        val str = assetId.toIdentifier()
        return firstOrNull { it.id().toIdentifier() == str }
    }

    private fun buildDetailElements(
        request: ConfirmParams?,
        assetsInfo: List<AssetInfo>?,
    ): List<ConfirmDetailElement> {
        return listOfNotNull(
            buildSwapDetailElement(request as? ConfirmParams.SwapParams, assetsInfo),
            buildPerpetualDetailElement(request as? ConfirmParams.PerpetualParams),
        )
    }

    private fun buildPerpetualDetailElement(
        params: ConfirmParams.PerpetualParams?,
    ): ConfirmDetailElement? = when (val type = params?.perpetualType) {
        null -> null
        is PerpetualType.Modify -> PerpetualModifyAutocloseFactory.create(type.content)
        else -> PerpetualConfirmDetailsUIModelFactory.create(type)?.let(ConfirmDetailElement::PerpetualDetails)
    }

    private fun buildSwapDetailElement(
        params: ConfirmParams.SwapParams?,
        assetsInfo: List<AssetInfo>?,
    ): ConfirmDetailElement.SwapDetails? {
        val params = params ?: return null
        val assetsInfo = assetsInfo ?: return null
        val fromAssetInfo = assetsInfo.getByAssetId(params.fromAsset.id) ?: return null
        val toAssetInfo = assetsInfo.getByAssetId(params.toAsset.id) ?: return null

        val provider = SwapProviderUIModelFactory.create(
            providerId = params.providerId,
            title = params.protocol,
            receiveAsset = toAssetInfo,
            toValue = params.toAmount.toString(),
        )
        val model = SwapDetailsUIModelFactory.create(
            SwapDetailsUIModelInput(
                payAsset = fromAssetInfo,
                receiveAsset = toAssetInfo,
                fromValue = params.fromAmount.toString(),
                toValue = params.toAmount.toString(),
                provider = provider,
                slippageBps = params.slippageBps,
                etaInSeconds = params.etaInSeconds,
                isProviderSelectable = false,
            )
        ) ?: return null

        return ConfirmDetailElement.SwapDetails(model)
    }
}
