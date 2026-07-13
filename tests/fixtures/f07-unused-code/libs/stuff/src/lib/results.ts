// Union members referenced only by the union type below. FetchResult is
// imported by the app, so its members are ALIVE — at most their `export`
// keyword is unnecessary.
export interface FetchSuccessResult {
  success: true;
}

export interface FetchErrorResult {
  success: false;
}

export type FetchResult = FetchSuccessResult | FetchErrorResult;

// Dead cluster: nobody imports DeadResult, so its member is dead too —
// internal references from dead exports must NOT revive it.
export interface DeadSuccessResult {
  success: true;
}

export type DeadResult = DeadSuccessResult;
