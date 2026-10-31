// Copyright (c). Gem Wallet. All rights reserved.

@testable import Components
import Style
import Testing

struct ButtonStateTests {
    @Test
    func errorState() {
        #expect(ButtonState(StateViewType<String>.error(TestError())) == .disabled)
        #expect(ButtonState(StateViewType<String>.error(TestError()), isDisabled: true) == .disabled)
        #expect(ButtonState(StateViewType<String>.error(TestError()), isDisabled: false) == .normal)
    }

    @Test
    func dataState() {
        #expect(ButtonState(StateViewType<String>.data("test")) == .normal)
        #expect(ButtonState(StateViewType<String>.data("test"), isDisabled: true) == .disabled)
        #expect(ButtonState(StateViewType<String>.data("test"), isDisabled: false) == .normal)
    }

    @Test
    func loadingState() {
        #expect(ButtonState(StateViewType<String>.loading) == .loading(showProgress: true))
        #expect(ButtonState(StateViewType<String>.loading, showProgress: false) == .loading(showProgress: false))
        #expect(ButtonState(StateViewType<String>.loading, isDisabled: true) == .disabled)
    }

    @Test
    func noDataState() {
        #expect(ButtonState(StateViewType<String>.noData) == .disabled)
        #expect(ButtonState(StateViewType<String>.noData, isDisabled: true) == .disabled)
        #expect(ButtonState(StateViewType<String>.noData, isDisabled: false) == .disabled)
    }
}

private struct TestError: Error {}
