"""Errors shared by controller, target and transport adapters."""
class BridgeError(Exception):
    def __init__(self, kind, message, certainty="NotExecuted"):
        super().__init__(message)
        self.kind, self.certainty = kind, certainty
