// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import class Gemstone.GemFeeService
import struct Gemstone.GemConfirmPreload
import struct Gemstone.GemConfirmSimulation
import struct Gemstone.GemConfirmLoadOptions
import enum Gemstone.GemTransferAmountResult
import Store
import GemstoneServices
import struct Gemstone.GemConfirmData
import enum Gemstone.GemConfirmError
import class Gemstone.GemTransferService
import protocol Gemstone.GemConfirmServiceProtocol
import enum Gemstone.GemExecuteResult
import protocol Gemstone.GemExplorerServiceProtocol
import protocol Gemstone.GemAssetsServiceProtocol
import protocol Gemstone.GemNameServiceProtocol
import protocol Gemstone.GemPerpetualServiceProtocol
import protocol Gemstone.GemPreferencesServiceProtocol
import struct Gemstone.GemSendInput
import protocol Gemstone.GemTransactionSigner
import protocol Gemstone.GemTransactionStateServiceProtocol
import GemstonePrimitives
import Validators
import Primitives
import PrimitivesComponents

public struct ConfirmService: Sendable {
    private let gemConfirmService: any GemConfirmServiceProtocol
    private let signer: any GemTransactionSigner
    private let preferencesService: any GemPreferencesServiceProtocol
    private let transactionStateService: any GemTransactionStateServiceProtocol
    private let recentAssetsService: any RecentAssetsServiceable
    private let toastPresenter: ToastPresenter
    private let keystore: any Keystore
    private let explorerService: any GemExplorerServiceProtocol
    private let nameService: any GemNameServiceProtocol
    private let assetsService: any GemAssetsServiceProtocol

    private let feeService: GemFeeService
    private let transferService: GemTransferService

    public let perpetualService: any GemPerpetualServiceProtocol

    public init(
        gemConfirmService: any GemConfirmServiceProtocol,
        signer: any GemTransactionSigner,
        preferencesService: any GemPreferencesServiceProtocol,
        transactionStateService: any GemTransactionStateServiceProtocol,
        recentAssetsService: any RecentAssetsServiceable,
        toastPresenter: ToastPresenter,
        keystore: any Keystore,
        explorerService: any GemExplorerServiceProtocol,
        nameService: any GemNameServiceProtocol,
        assetsService: any GemAssetsServiceProtocol,
        feeService: GemFeeService,
        transferService: GemTransferService,
        perpetualService: any GemPerpetualServiceProtocol,
    ) {
        self.perpetualService = perpetualService
        self.gemConfirmService = gemConfirmService
        self.signer = signer
        self.preferencesService = preferencesService
        self.transactionStateService = transactionStateService
        self.recentAssetsService = recentAssetsService
        self.toastPresenter = toastPresenter
        self.keystore = keystore
        self.explorerService = explorerService
        self.nameService = nameService
        self.assetsService = assetsService
        self.feeService = feeService
        self.transferService = transferService
    }

    func simulationState(request: ConfirmTransferRequest) -> ConfirmSimulationState {
        makeSimulationState(data: request.data, simulation: request.simulation, addressNames: [:])
    }

    func metadata(request: ConfirmTransferRequest) throws -> TransferDataMetadata {
        let assetId = request.data.type.asset.id
        let feeAssetId = request.data.type.feeAsset(transferService: transferService).id
        return try gemConfirmService.metadata(
            walletId: request.wallet.id.id,
            assetId: assetId.identifier,
            feeAssetId: feeAssetId.identifier,
            extraAssetIds: request.data.type.assetIds(transferService: transferService).map(\.identifier),
        ).map(assetId: assetId, feeAssetId: feeAssetId)
    }

    func load(request: ConfirmTransferRequest, selection: FeeSelection, feeAssetSelection: FeeAssetSelection) async throws -> ConfirmTransferData {
        let feeAssets = try gemConfirmService.feeAssets(walletId: request.wallet.id.id, chain: request.data.chain.rawValue)
        let preload = try await preload(request: request, selection: selection, feeAssetSelection: feeAssetSelection)
        return ConfirmTransferData(
            preload: preload,
            simulation: await updatedSimulationState(data: request.data, simulation: request.simulation ?? preload.simulation),
            feeAssets: feeAssets,
        )
    }

    func preload(request: ConfirmTransferRequest, selection: FeeSelection, feeAssetSelection: FeeAssetSelection) async throws -> ConfirmTransferPreload {
        let metadata = try metadata(request: request)
        let account = try request.wallet.account(for: request.data.chain)
        let preload: GemConfirmPreload
        do {
            preload = try await gemConfirmService.preload(
                walletId: request.wallet.id.id,
                input: request.data.confirmInput(from: account),
                options: GemConfirmLoadOptions(
                    feeSelection: selection.map(),
                    feeAssetId: feeAssetSelection.selectedAssetId?.identifier,
                ),
            )
        } catch let error as GemConfirmError {
            throw preloadFailureError(metadata: metadata) ?? error
        }
        let feeAsset = try Asset(preload.feeAsset)
        return try ConfirmTransferPreload(
            metadata: preload.metadata.map(assetId: metadata.assetId, feeAssetId: feeAsset.id),
            input: ConfirmTransferInput(
                confirmData: preload.confirmData,
                fee: preload.confirmData.fee.map(),
                transferAmount: transferAmount(preload.amount, asset: request.data.type.asset, feeAsset: feeAsset),
                feeAsset: feeAsset,
            ),
            feeRates: preload.confirmData.feeRates.map { try $0.map() },
            simulation: preload.confirmData.simulation.map { try Primitives.SimulationResult($0) },
        )
    }

    private func preloadFailureError(metadata: TransferDataMetadata) -> TransferAmountCalculatorError? {
        guard feeService.isInsufficientNetworkFee(feeAssetId: metadata.feeAssetId.identifier, feeAvailable: metadata.feeAvailable.description) else {
            return nil
        }
        return .insufficientNetworkFee(metadata.feeAssetId.chain.asset, requirement: nil)
    }

    private func transferAmount(_ result: GemTransferAmountResult, asset: Asset, feeAsset: Asset) -> TransferAmountValidation {
        switch result {
        case let .amount(amount):
            .success(TransferAmount(
                value: (try? BigInt.from(string: amount.value)) ?? .zero,
                networkFee: (try? BigInt.from(string: amount.networkFee)) ?? .zero,
                useMaxAmount: amount.isMaxAmount,
            ))
        case let .error(error):
            .failure(TransferAmountCalculatorError(
                (try? error.map()) ?? .insufficientBalance(
                    assetId: error.assetId ?? asset.id,
                    requirement: BalanceRequirement(required: .zero, available: .zero),
                ),
                asset: asset,
                assetFee: feeAsset,
            ))
        }
    }

    func confirm(request: ConfirmTransferRequest, confirmData: GemConfirmData, amount: TransferAmount, simulation: SimulationResult?) async throws {
        let input = GemSendInput(
            wallet: request.wallet.json(),
            confirm: confirmData,
            value: amount.value.description,
            networkFee: amount.networkFee.description,
            simulation: simulation?.json(),
        )
        let result: GemExecuteResult
        do {
            result = try await gemConfirmService.execute(input: input, signer: signer)
        } catch let GemConfirmError.Broadcast(hashes, msg) {
            hashes.forEach { request.delegate?(.success($0)) }
            trackPending()
            throw GemConfirmError.Broadcast(hashes: hashes, msg: msg)
        }
        switch result {
        case let .signed(data):
            data.forEach { request.delegate?(.success($0)) }
        case let .sent(hashes, transactions):
            hashes.forEach { request.delegate?(.success($0)) }
            track(wallet: request.wallet, transactions: try transactions.map { try Transaction($0) })
        }
        await toastPresenter.present(.transfer(for: request.data.type))
        if let recent = request.data.type.recentActivityData {
            updateRecent(data: recent, walletId: request.wallet.id)
        }
    }

    private func trackPending() {
        Task {
            do {
                try await transactionStateService.trackPending()
            } catch {
                debugLog("confirm: pending tracking failed \(error)")
            }
        }
    }

    private func track(wallet: Wallet, transactions: [Transaction]) {
        Task {
            do {
                try await transactionStateService.track(walletId: wallet.id.id, transactions: transactions.map { $0.json() })
            } catch {
                debugLog("confirm: transaction tracking failed \(error)")
            }
        }
    }

    public func explorerLink(chain: Chain, address: String) -> BlockExplorerLink {
        BlockExplorerLink(explorerService.getAddressUrl(chain: chain.rawValue, address: address))
    }

    public func addressName(chain: Chain, address: String) throws -> AddressName? {
        try nameService.addressName(chain: chain.rawValue, address: address).map { try AddressName($0) }
    }

    public var currency: Currency {
        preferencesService.currencyValue
    }

    public func passwordAuthentication() throws -> KeystoreAuthentication {
        try keystore.getPasswordAuthentication()
    }

    public func defaultPriority(for type: TransferDataType) -> FeePriority {
        (try? type.map()).map { feeService.defaultPriority(inputType: $0).map() } ?? .normal
    }
}

// MARK: - Private

private extension ConfirmService {
    func updateRecent(data: RecentActivityData, walletId: WalletId) {
        do {
            try recentAssetsService.add(data, walletId: walletId)
        } catch {
            debugLog("Failed to update recent activity: \(error)")
        }
    }
}

// MARK: - Simulation

private extension ConfirmService {
    func makeSimulationState(data: TransferData, simulation: SimulationResult?, addressNames: [ChainAddress: AddressName]) -> ConfirmSimulationState {
        let resolved = try? gemConfirmService.simulation(inputType: data.type.inputType, simulation: simulation?.json())
        let fields = resolved?.payloadFields.compactMap { try? SimulationPayloadField($0) } ?? []
        var payload = SimulationPayloadModel(
            chain: data.chain,
            primaryFields: fields.primaryFields,
            secondaryFields: fields.secondaryFields,
        )
        payload.addressNames = addressNames
        return ConfirmSimulationState(
            result: simulation,
            warnings: simulation?.warnings ?? [],
            payload: payload,
            headerData: resolved?.header.flatMap { header in
                (try? Asset(header.asset)).map { AssetValueHeaderData(asset: $0, value: header.value.map()) }
            },
            balanceChanges: resolved?.balanceChanges.compactMap { change in
                guard let asset = try? Asset(change.asset), let value = BigInt(change.value, radix: 10) else { return nil }
                return SimulationAssetChange(asset: asset, value: value)
            } ?? [],
        )
    }

    func updatedSimulationState(data: TransferData, simulation: SimulationResult?) async -> ConfirmSimulationState {
        do {
            try await assetsService.syncMissingAssets(for: simulation?.simulationAssetIds ?? [])
        } catch {
            debugLog("simulation asset preload error: \(error)")
        }
        let requests = makeSimulationState(data: data, simulation: simulation, addressNames: [:]).payload.addressRequests
        let names = (try? await nameService.addressNames(requests: requests)) ?? [:]
        return makeSimulationState(data: data, simulation: simulation, addressNames: names)
    }
}

private extension SimulationResult {
    var simulationAssetIds: [AssetId] {
        balanceChanges.map(\.assetId) + [header?.assetId].compactMap(\.self)
    }
}
