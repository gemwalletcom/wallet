// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSignMessagePreview
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
import struct Gemstone.SimulationPayloadField
import struct Gemstone.GemSimulationWarningRow
import func Gemstone.simulationWarningRows

@Observable
@MainActor
public final class SignMessageSceneViewModel {
    private let service: any GemSignMessageServiceProtocol
    private let payload: SignMessagePayload
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate
    private let preview: GemSignMessagePreview
    private let row: GemWalletRow

    public var isPresentingUrl: URL?
    public var isPresentingPayloadDetails: Bool = false
    public var isPresentingAlertMessage: AlertMessage?
    private var payloadAddressNames: [ChainAddress: AddressName] = [:]

    public init(
        service: any GemSignMessageServiceProtocol,
        payload: SignMessagePayload,
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
    ) {
        self.service = service
        self.payload = payload
        self.confirmTransferDelegate = confirmTransferDelegate
        row = walletRow(wallet: payload.wallet.toGem())
        preview = service.preview(message: payload.message, simulation: payload.simulation, assets: payload.assets.map { $0.toGem() })
    }

    var viewFullMessageListItem: ListItemModel {
        ListItemModel(title: Localized.SignMessage.viewFullMessage)
    }

    var payloadDetailsListItem: ListItemModel {
        ListItemModel(title: Localized.Common.details)
    }

    public var networkText: String {
        payload.chain.networkName
    }

    public var title: String {
        preview.messageType.title
    }

    public var walletText: String {
        payload.wallet.name
    }

    public var buttonTitle: String {
        Localized.Transfer.confirm
    }

    public var appName: String {
        payload.session.metadata.shortName
    }

    public var appAssetImage: AssetImage {
        AssetImage(imageURL: payload.session.metadata.iconURL)
    }

    public var walletAssetImage: AssetImage {
        row.avatarImage
    }

    public var networkAssetImage: AssetImage {
        AssetIdViewModel(assetId: payload.chain.asset.id).networkAssetImage
    }

    public var appText: String {
        appName
    }

    public var appPreview: AppPreviewModel {
        AppPreviewModel(
            assetImage: appAssetImage,
            name: appName,
            subtitleSymbol: payload.session.metadata.host,
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
        simulationWarningRows(warnings: payload.simulation.warnings)
    }

    public var payloadModel: SimulationPayloadModel {
        SimulationPayloadModel(
            chain: payload.chain,
            primaryFields: preview.primaryFields,
            secondaryFields: preview.secondaryFields,
            addressNames: payloadAddressNames,
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
        let signature = try await service.sign(walletId: payload.wallet.id.id, message: payload.message)
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

    func fieldModels(for fields: [SimulationPayloadField]) -> [SimulationPayloadFieldViewModel] {
        payloadModel.fieldModels(
            for: fields,
            explorerLink: { service.addressUrl(chain: payload.chain.rawValue, address: $0).toPrimitives() },
            onOpenURL: { [weak self] in self?.isPresentingUrl = $0 },
        )
    }

    func onViewPayloadDetails() {
        isPresentingPayloadDetails = true
    }
}

private extension SignMessageSceneViewModel {
    func loadPayloadAddressNamesIfNeeded() async {
        guard payloadAddressNames.isEmpty, payloadModel.hasFields else { return }

        payloadAddressNames = await service.addressNames(chain: payload.chain, preview: preview)
    }
}
