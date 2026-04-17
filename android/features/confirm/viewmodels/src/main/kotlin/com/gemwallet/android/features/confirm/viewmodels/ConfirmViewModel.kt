package com.gemwallet.android.features.confirm.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.confirm.coordinators.BuildConfirmProperties
import com.gemwallet.android.application.confirm.coordinators.ConfirmTransaction
import com.gemwallet.android.application.confirm.coordinators.ValidateBalance
import com.gemwallet.android.blockchain.services.SignerPreloaderProxy
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.transactions.TransactionBalanceService
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.GemPlatformErrors
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.model.format
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelInput
import com.gemwallet.android.ui.models.swap.SwapProviderUIModelFactory
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.domains.confirm.AmountUIModel
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Currency
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
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import com.wallet.core.primitives.SimulationResult
import java.math.BigInteger
import javax.inject.Inject

internal const val paramsArg = "data"
internal const val txTypeArg = "tx_type"

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
    private val savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val restart = MutableStateFlow(false)
    val state = MutableStateFlow<ConfirmState>(ConfirmState.Prepare)
    val feePriority = MutableStateFlow(FeePriority.Normal)
    private val walletConnectSimulationState = MutableStateFlow<SimulationResult?>(null)

    private val request = savedStateHandle.getStateFlow<String?>(paramsArg, null)
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
        val review = simulation?.toWalletConnectReview() ?: WalletConnectReview()
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
                ConfirmState.Error(
                    when (err.message?.contains(GemPlatformErrors.Dust.message)) {
                        true -> ConfirmError.DustThreshold(owner.chain)
                        else -> ConfirmError.PreloadError
                    }
                )
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
        val price = assetInfo.price?.price?.price ?: 0.0
        val currency = assetInfo.price?.currency ?: Currency.USD
        val decimals = assetInfo.asset.decimals
        val symbol = assetInfo.asset.symbol

        AmountUIModel(
            txType = request.getTxType(),
            amount = amount.format(decimals, symbol, -1),
            amountEquivalent = currency.format(amount.convert(decimals, price).atomicValue, dynamicPlace = true),
            asset = assetInfo,
            fromAsset = assetInfo,
            fromAmount = amount.atomicValue.toString(),
            toAsset = toAssetInfo,
            toAmount = (request as? ConfirmParams.SwapParams)?.toAmount?.toString(),
            nftAsset = (request as? ConfirmParams.NftParams)?.nftAsset,
            currency = currency,
        )
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val detailElements = combine(request, assetsInfo, ::buildDetailElements)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val txProperties = combine(request, assetsInfo) { request, assetsInfo ->
        request ?: return@combine emptyList()
        assetsInfo ?: return@combine emptyList()
        buildConfirmProperties(request, assetsInfo)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val feeValue = combine(preloadData, feeAssetInfo) { signerParams, feeAssetInfo ->
        val amount = signerParams?.fee()?.amount
        if (amount == null || feeAssetInfo == null) {
            return@combine ""
        }
        val feeAmount = Crypto(amount)
        feeAssetInfo.asset.format(feeAmount, 8, dynamicPlace = true)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val feeUIModel = combine(preloadData, feeAssetInfo, state) { signerParams, feeAssetInfo, state ->
        val amount = signerParams?.fee()?.amount
        val result = if (state is ConfirmState.Prepare) {
            FeeUIModel.Calculating
        } else if (amount == null || feeAssetInfo == null) {
            if (state is ConfirmState.Error) FeeUIModel.Error else FeeUIModel.Calculating
        } else {
            val feeAmount = Crypto(amount)
            val currency = feeAssetInfo.price?.currency ?: Currency.USD
            val feeDecimals = feeAssetInfo.asset.decimals
            val feeCrypto = feeAssetInfo.asset.format(feeAmount, 8, dynamicPlace = true)
            val feeFiat = feeAssetInfo.price?.let {
                currency.format(feeAmount.convert(feeDecimals, it.price.price).atomicValue, dynamicPlace = true) // TODO: Move to UI - Model
            } ?: ""

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
                cryptoAmount = feeCrypto,
                fiatAmount = feeFiat,
                feeAsset = feeAssetInfo.asset,
                priority = signerParams.fee().priority,
            )
        }
        result
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeRates = preloadData.map { it?.feeRates.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun init(params: ConfirmParams, walletConnectSimulation: SimulationResult? = null) {
        viewModelScope.launch(Dispatchers.IO) {
            state.update { ConfirmState.Prepare }
            walletConnectSimulationState.value = walletConnectSimulation
            // reset
            savedStateHandle[txTypeArg] = null
            savedStateHandle[paramsArg] = null
            // load
            savedStateHandle[txTypeArg] = params.getTxType().string
            savedStateHandle[paramsArg] = params.pack()
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
            val result = confirmTransaction(signerParams, session, assetInfo, viewModelScope)
            state.update { ConfirmState.Result(txHash = result.txHash) }
            viewModelScope.launch(Dispatchers.Main) {
                finishAction(assetId = assetInfo.id(), hash = result.txHash, route = result.finishRoute)
            }
        } catch (err: ConfirmError) {
            state.update { ConfirmState.BroadcastError(err) }
        } catch (err: Throwable) {
            state.update { ConfirmState.BroadcastError(ConfirmError.BroadcastError(err.message ?: "Unknown error")) }
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
        )
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
