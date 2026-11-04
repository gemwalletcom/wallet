import Components
import Foundation
import struct Gemstone.GemInfoSheet
import enum Gemstone.GemInfoTopic
import protocol Gemstone.GemNameServiceProtocol
import enum Gemstone.GemWalletImportKind
import struct Gemstone.GemWalletImportScreen
import protocol Gemstone.GemWalletServiceProtocol
import func Gemstone.phraseSuggestions
import GemstonePrimitives
import GemstoneServices
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
final class ImportWalletSceneViewModel {
    private let service: any GemWalletServiceProtocol
    let type: ImportWalletType
    private let importScreen: GemWalletImportScreen

    var inputModel = ImportInputViewModel()

    var importType: GemWalletImportKind = .phrase {
        didSet {
            inputModel.update(text: "")
            isImporting = false
        }
    }

    private var isImporting = false
    let nameRecordViewModel: NameRecordViewModel

    var isPresentingScanner = false
    var isPresentingAlertMessage: AlertMessage?
    var isPresentingExistingWallet: GemInfoSheet?

    private let onComplete: VoidAction

    init(
        service: any GemWalletServiceProtocol,
        nameService: any GemNameServiceProtocol,
        type: ImportWalletType,
        onComplete: VoidAction,
    ) {
        self.service = service
        self.type = type
        importScreen = service.importScreen(chain: type.chain?.toGem())
        self.onComplete = onComplete
        nameRecordViewModel = NameRecordViewModel(nameService: nameService)
    }

    var title: String {
        importScreen.title.text
    }

    var wordsSuggestion: [String] {
        guard importType.supportsPhraseSuggestions(), inputModel.isTypingLastWord else { return [] }
        return phraseSuggestions(word: inputModel.lastWord)
    }

    var showsPhraseSuggestions: Bool {
        wordsSuggestion.isNotEmpty
    }

    var buttonState: ButtonState {
        isImporting ? .loading(showProgress: true) : .normal
    }

    var pasteButtonTitle: String {
        Localized.Common.paste
    }

    var pasteButtonImage: Image {
        Images.System.paste
    }

    var qrButtonTitle: String {
        Localized.Wallet.scan
    }

    var qrButtonImage: Image {
        Images.System.qrCodeViewfinder
    }

    var alertTitle: String {
        Localized.Errors.validation("")
    }

    var showImportTypes: Bool {
        importScreen.showsKinds
    }

    var importTypes: [GemWalletImportKind] {
        importScreen.kinds
    }

    var footerText: String? {
        importType.showsViewOnlyWarning() ? Localized.Wallet.importAddressWarning : nil
    }

    var shouldProtectInput: Bool {
        importType.protectsInput()
    }

    var showsNameRecord: Bool {
        importType.resolvesNames()
    }

    var highlightsWords: Bool {
        importType == .phrase
    }
}

// MARK: - Business Logic

extension ImportWalletSceneViewModel {
    func onChangeInput(_: String, newValue: String) {
        if showsNameRecord, let chain = type.chain {
            nameRecordViewModel.getNameRecord(name: newValue, chain: chain)
        } else {
            nameRecordViewModel.reset()
        }
    }

    func onSelectActionButton() async {
        isImporting = true

        do {
            try await importWallet()
        } catch {
            isImporting = false
            inputModel.update(error: error)
            isPresentingAlertMessage = AlertMessage(title: alertTitle, error: error)
        }
    }

    func onSelectScanQR() {
        isPresentingScanner = true
    }

    func onHandleScan(_ result: String) {
        inputModel.update(text: result)
    }

    func onSelectWord(_ word: String) {
        inputModel.complete(word: word)
    }

    func onPaste() {
        guard let string = UIPasteboard.general.string else {
            UINotificationFeedbackGenerator().notificationOccurred(.error)
            return
        }
        inputModel.update(text: string.trim())

        if shouldProtectInput {
            Clipboard.clear()
        }
    }

    func onSelectExistingWalletContinue() {
        isPresentingExistingWallet = nil
        onComplete?()
    }
}

// MARK: - Private

extension ImportWalletSceneViewModel {
    private func importWallet() async throws {
        let result = try await service.importWallet(
            kind: importType,
            chain: type.chain,
            input: inputModel.text,
            nameRecord: showsNameRecord ? nameRecordViewModel.state.record() : nil,
            source: .import,
        )
        isImporting = false
        switch result {
        case .new: onComplete?()
        case let .existing(wallet): isPresentingExistingWallet = GemInfoTopic.existingWalletImported(name: wallet.name).infoSheet
        }
    }
}
