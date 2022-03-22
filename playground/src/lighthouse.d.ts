// Modified from https://github.com/GoogleChrome/lighthouse/pull/13523

declare module "lighthouse" {

export = lighthouse;

type Result = {
  report: string,
  lhr: {
    finalUrl: string,
    categories: {performance: {score: number}},
  },
};

declare function lighthouse(url?: string, flags?: any): Promise<Result | undefined>;

}  // declare module "lighthouse"
