// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSignMessagePreview
import struct Gemstone.GemWalletConnectMessageRequest
import struct Gemstone.GemWalletRow
import func Gemstone.walletRow
import protocol Gemstone.GemSignMessageServiceProtocol
import Components
import Foundation
import struct Gemstone.GemSimulationValue
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import WalletConnectorService
import struct Gemstone.GemSimulationPayloadRow
import struct Gemstone.GemSimulationWarningRow
import func Gemstone.simulationWarningRows

@Observable
@MainActor
public final class SignMessageSceneViewModel {
    private let service: any GemSignMessageServiceProtocol
    private let request: GemWalletConnectMessageRequest
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate
    private var preview: GemSignMessagePreview
    private let row: GemWalletRow

    public var isPresentingUrl: URL?
    public var isPresentingPayloadDetails: Bool = false
    public var isPresentingAlertMessage: AlertMessage?
    private var hasLoadedAddressNames = false

    public init(
        service: any GemSignMessageServiceProtocol,
        request: GemWalletConnectMessageRequest,
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
    ) {
        self.service = service
        self.request = request
        self.confirmTransferDelegate = confirmTransferDelegate
        row = walletRow(wallet: request.wallet)
        preview = service.preview(message: request.message, simulation: request.simulation, assets: request.assets)
    }

    private var chain: Chain {
        Chain(core: request.chain)
    }

    private var metadata: ApplicationMetadata {
        request.session.metadata.toPrimitives()
    }

    var viewFullMessageListItem: ListItemModel {
        ListItemModel(title: Localized.SignMessage.viewFullMessage)
    }

    var payloadDetailsListItem: ListItemModel {
        ListItemModel(title: Localized.Common.details)
    }

    public var networkText: String {
        chain.networkName
    }

    public var title: String {
        preview.messageType.title
    }

    public var walletText: String {
        request.wallet.name
    }

    public var buttonTitle: String {
        Localized.Transfer.confirm
    }

    public var appName: String {
        metadata.shortName
    }

    public var appAssetImage: AssetImage {
        AssetImage(imageURL: metadata.iconURL)
    }

    public var walletAssetImage: AssetImage {
        row.avatarImage
    }

    public var networkAssetImage: AssetImage {
        AssetIdViewModel(assetId: chain.asset.id).networkAssetImage
    }

    public var appText: String {
        appName
    }

    public var appPreview: AppPreviewModel {
        AppPreviewModel(
            assetImage: appAssetImage,
            name: appName,
            subtitleSymbol: metadata.host,
        )
    }

    public var headerModel: AssetValueHeaderViewModel? {
        headerData.map { AssetValueHeaderViewModel(data: $0) }
    }

    public var headerData: GemSimulationValue? {
        preview.header
    }

    var messageText: String {
        preview.text
    }

    var textMessageViewModel: TextMessageViewModel {
        TextMessageViewModel(message: preview.text)
    }

    public var simulationWarningModels: [SimulationWarningViewModel] {
        simulationWarnings.map(SimulationWarningViewModel.init)
    }

    public var simulationWarnings: [GemSimulationWarningRow] {
        simulationWarningRows(warnings: request.simulation.warnings)
    }

    public var payloadModel: SimulationPayloadModel {
        SimulationPayloadModel(
            primaryFields: preview.primaryFields,
            secondaryFields: preview.secondaryFields,
        )
    }

    public var hasWarnings: Bool {
        !simulationWarnings.isEmpty
    }

    public var isButtonDisabled: Bool {
        preview.hasCriticalWarning
    }

    public var buttonType: ButtonType {
        .primary(isButtonDisabled ? .disabled : .normal)
    }

    public func signMessage() async throws {
        let signature = try await service.sign(walletId: request.wallet.id, message: request.message)
        confirmTransferDelegate(.success(signature))
    }

    public func onSign(onComplete: @escaping () -> Void) {
        Task {
            do {
                try await signMessage()
                onComplete()
            } catch {
                isPresentingAlertMessage = AlertMessage(error: error)
            }
        }
    }
}

// MARK: - Actions

public extension SignMessageSceneViewModel {
    func load() {
        Task {
            await loadPayloadAddressNamesIfNeeded()
        }
    }

    func fieldModels(for fields: [GemSimulationPayloadRow]) -> [SimulationPayloadFieldViewModel] {
        payloadModel.fieldModels(
            for: fields,
            explorerLink: { service.addressUrl(chain: request.chain, address: $0).toPrimitives() },
            onOpenURL: { [weak self] in self?.isPresentingUrl = $0 },
        )
    }

    func onViewPayloadDetails() {
        isPresentingPayloadDetails = true
    }
}

private extension SignMessageSceneViewModel {
    func loadPayloadAddressNamesIfNeeded() async {
        guard !hasLoadedAddressNames, payloadModel.hasFields else { return }

        hasLoadedAddressNames = true
        preview = await service.withAddressNames(chain: request.chain, preview: preview)
    }
}
