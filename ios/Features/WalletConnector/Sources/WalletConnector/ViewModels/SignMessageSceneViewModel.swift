// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemListRow
import enum Gemstone.GemServiceError
import struct Gemstone.GemSignMessagePreview
import protocol Gemstone.GemSignMessageServiceProtocol
import struct Gemstone.GemSimulationPayloadRow
import struct Gemstone.GemSimulationValue
import struct Gemstone.GemWalletConnectMessageRequest
import func Gemstone.signerFailure
import func Gemstone.simulationWarningRows
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import WalletConnectorService

@Observable
@MainActor
public final class SignMessageSceneViewModel {
    private let service: any GemSignMessageServiceProtocol
    private let request: GemWalletConnectMessageRequest
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate
    private var preview: GemSignMessagePreview

    public var presentedSheet: SignMessageSheet?
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
        preview = service.preview(request: request)
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

    public var title: String {
        preview.messageType.title
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

    var rows: [GemListRow] {
        preview.rows
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

    public var simulationWarnings: [GemListRow] {
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
            } catch let error as GemServiceError {
                switch signerFailure(error: error.text()) {
                case let .retry(text):
                    isPresentingAlertMessage = AlertMessage(title: Localized.Errors.errorOccurred, message: text.text)
                case .reject:
                    confirmTransferDelegate(.failure(error))
                    onComplete()
                }
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
            onSelectAddress: { [weak self] address in
                guard let self else { return }
                presentedSheet = .addressDetails(ChainAddress(chain: Chain(core: request.chain), address: address))
            },
        )
    }

    func onViewPayloadDetails() {
        presentedSheet = .payloadDetails
    }
}

private extension SignMessageSceneViewModel {
    func loadPayloadAddressNamesIfNeeded() async {
        guard !hasLoadedAddressNames, payloadModel.hasFields else { return }

        hasLoadedAddressNames = true
        preview = await service.withAddressNames(chain: request.chain, preview: preview)
    }
}
