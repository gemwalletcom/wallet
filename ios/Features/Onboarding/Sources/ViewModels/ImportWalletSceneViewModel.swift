import Components
import Foundation
import protocol Gemstone.GemNameServiceProtocol
import enum Gemstone.GemWalletImportKind
import struct Gemstone.GemWalletImportScreen
import protocol Gemstone.GemWalletServiceProtocol
import func Gemstone.phraseSuggestions
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
final class ImportWalletSceneViewModel {
    private let service: any GemWalletServiceProtocol
    let preferences: ObservablePreferences
    let type: ImportWalletType
    private let importScreen: GemWalletImportScreen

    var input: String = "" {
        didSet { refreshSuggestions() }
    }

    var inputCursor: Int? {
        didSet { refreshSuggestions() }
    }

    var importType: GemWalletImportKind = .phrase {
        didSet {
            input = ""
            inputCursor = nil
            isImporting = false
        }
    }

    private(set) var wordsSuggestion: [String] = []
    private var isImporting = false
    let nameRecordViewModel: NameRecordViewModel

    var isPresentingScanner = false
    var isPresentingAlertMessage: AlertMessage?
    var isPresentingExistingWalletName: String?

    private let onComplete: VoidAction

    init(
        service: any GemWalletServiceProtocol,
        preferences: ObservablePreferences,
        nameService: any GemNameServiceProtocol,
        type: ImportWalletType,
        onComplete: VoidAction,
    ) {
        self.service = service
        self.preferences = preferences
        self.type = type
        importScreen = switch type {
        case .multicoin: service.importScreen(chain: nil)
        case let .chain(chain): service.importScreen(chain: chain.toGem())
        }
        self.onComplete = onComplete
        nameRecordViewModel = NameRecordViewModel(nameService: nameService)
    }

    var title: String {
        importScreen.title.text
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

    var chain: Chain? {
        switch type {
        case .multicoin: .none
        case let .chain(chain): chain
        }
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

    var docsUrl: URL {
        AppUrl.docs(.howToSecureSecretPhrase)
    }

    var shouldProtectInput: Bool {
        importType.protectsInput()
    }

    var showsNameRecord: Bool {
        importType.resolvesNames()
    }
}

// MARK: - Business Logic

extension ImportWalletSceneViewModel {
    func onChangeInput(_: String, newValue: String) {
        if showsNameRecord, let chain {
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
            isPresentingAlertMessage = AlertMessage(title: alertTitle, error: error)
        }
    }

    func onSelectScanQR() {
        isPresentingScanner = true
    }

    func onHandleScan(_ result: String) {
        input = result
        inputCursor = nil
    }

    func onSelectWord(_ word: String) {
        input = String(input.dropLast(lastWord.count)) + word + " "
        inputCursor = input.utf16.count
    }

    func onPaste() {
        guard let string = UIPasteboard.general.string else {
            UINotificationFeedbackGenerator().notificationOccurred(.error)
            return
        }
        input = string.trim()
        inputCursor = nil

        if shouldProtectInput {
            CopyTypeViewModel.clearClipboard()
        }
    }

    func onSelectExistingWalletContinue() {
        onComplete?()
    }
}

// MARK: - Private

extension ImportWalletSceneViewModel {
    private func importWallet() async throws {
        let result = try await service.importWallet(
            kind: importType,
            chain: chain,
            input: input,
            nameRecord: showsNameRecord ? nameRecordViewModel.state.record() : nil,
            source: .import,
        )
        isImporting = false
        switch result {
        case .new: onComplete?()
        case let .existing(wallet): isPresentingExistingWalletName = wallet.name
        }
    }
}

extension ImportWalletSceneViewModel {
    private var lastWord: String {
        input.split(omittingEmptySubsequences: false, whereSeparator: \.isWhitespace).last.map(String.init) ?? ""
    }

    private var isTypingLastWord: Bool {
        inputCursor.map { $0 >= input.utf16.count } ?? true
    }

    private func refreshSuggestions() {
        guard importType.supportsPhraseSuggestions(), isTypingLastWord else {
            wordsSuggestion = []
            return
        }
        wordsSuggestion = phraseSuggestions(word: lastWord)
    }
}
