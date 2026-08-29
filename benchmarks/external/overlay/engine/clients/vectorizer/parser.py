"""Filter parsing for filtered-search datasets.

Not implemented yet. The framework only reaches a parser for datasets that
carry `metaconditions`; the unfiltered ANN datasets (glove, dbpedia, laion)
never call it. Raising here rather than returning `None` keeps a filtered
dataset from running unfiltered and reporting recall against ground truth it
never applied the filter for — which would look like an excellent result.
"""

from engine.base_client.parser import BaseConditionParser, FieldValue


class VectorizerConditionParser(BaseConditionParser):
    def build_condition(self, and_subfilters, or_subfilters):
        raise NotImplementedError(_UNSUPPORTED)

    def build_exact_match_filter(self, field_name: str, value: FieldValue):
        raise NotImplementedError(_UNSUPPORTED)

    def build_range_filter(self, field_name: str, lt, gt, lte, gte):
        raise NotImplementedError(_UNSUPPORTED)

    def build_geo_filter(self, field_name: str, lat: float, lon: float, radius: float):
        raise NotImplementedError(_UNSUPPORTED)

    def build_radius_filter(self, field_name: str, lat, lon, radius):
        raise NotImplementedError(_UNSUPPORTED)

    def build_match_filter(self, field_name: str, value: str):
        raise NotImplementedError(_UNSUPPORTED)

    def build_any_filter(self, field_name: str, value):
        raise NotImplementedError(_UNSUPPORTED)


_UNSUPPORTED = (
    "The Vectorizer benchmark client does not implement filtered search yet. "
    "Run it against unfiltered datasets, or implement this against the "
    "server's filter syntax — do not stub it out to return no filter, which "
    "would score an unfiltered search against filtered ground truth."
)

__all__ = ["VectorizerConditionParser", "FieldValue"]
