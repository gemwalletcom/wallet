// Copyright (c). Gem Wallet. All rights reserved.

import XCTest

@MainActor
final class CreateWalletUITests: XCTestCase {
    override func setUpWithError() throws {
        try super.setUpWithError()
        continueAfterFailure = false
    }

    func testCreateWalletAndReceiveBitcoin() {
        let app = XCUIApplication()
        setupPermissionHandler()
        app.launch()
        app.logout()

        if app.isOnboarding {
            app.tapCreateWallet()
        }

        let words = creationFlow(app: app, checkShowSecretDataScene: true)

        app.tapWalletBar()

        app.buttons["gearshape"].firstMatch.tap()

        app.buttons["Show Secret Phrase"].firstMatch.tap()

        app.tapContinue()

        let displayedWords = app.getWords()
        XCTAssertEqual(words, displayedWords)

        app.tapBack()
        app.tapBack()
        app.tapBack()

        app.tapCreateWallet()

        _ = creationFlow(app: app, checkShowSecretDataScene: false)
    }

    private func creationFlow(app: XCUIApplication, checkShowSecretDataScene: Bool) -> [String] {
        app.acceptTerms()

        app.tapContinue()

        let words = app.getWords()
        app.tapContinue()

        if checkShowSecretDataScene {
            app.tapBack()
            words.forEach { XCTAssert(app.staticTexts[$0].exists) }
        }
        app.tapContinue()

        words.forEach { app.buttons[$0].firstMatch.tap() }
        app.tapContinue()
        app.skipEnableAuthentication()

        return words
    }
}
