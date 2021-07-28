import sys
import re
from Rule import Rule
from Expression import Expression
from Stack import Stack
import csv
from joblib import Parallel, delayed
import multiprocessing


def extract(path, delimiter):
    num_cores = multiprocessing.cpu_count() // 2
    with open(path) as csv_file:
        csv_reader = csv.reader(csv_file, delimiter=delimiter)
        remove = ['int32', 'float32', 'select', 'broadcast', 'ramp', 'fold', 'Overflow',
                  'can_prove', 'canprove', 'op->type', 'op->type', 'Call', 'this', 'IRMatcher']
        exprs = []
        exprs = Parallel(n_jobs=num_cores)(delayed(extract_one)(
            i, row, remove) for i, row in enumerate(csv_reader))
        # for i, row in enumerate(csv_reader):
        #     exprs.append(extract_one(i, row, remove))
    return exprs


def extract_one(i, row, remove):
    try:
        if len(row[0]) < 1000:
            next_expr = False
            for tabou in remove:
                if tabou in row[0]:
                    # print("=====", tabou)
                    next_expr = True
            if next_expr:
                # print("Skipped row :", i)
                return None
            row[0] = row[0].replace("(uint1)", "")
            right = Expression(row[0])
            expr = ' '.join(right.infixToPrefix())
            expr = re.sub(
                "\( \- (?P<var>[a-zA-Z_$][a-zA-Z_$0-9]*) \)", r'(* \1 -1)', expr)

            row[1] = row[1].replace("(uint1)", "")
            halideResult = Expression(row[1])
            halideResult = ' '.join(halideResult.infixToPrefix())
            halideResult = re.sub(
                "\( \- (?P<var>[a-zA-Z_$][a-zA-Z_$0-9]*) \)", r'(* \1 -1)', halideResult)
            print("Expression " + str(i) + " processed.")
            return (expr, halideResult, row[2])
        else:
            raise Exception("Expression too long!")
    except Exception as e:
        print(e)
        print("Expression " + str(i) + " skipped.")


if __name__ == '__main__':
    if len(sys.argv) > 2:
        delimiter = sys.argv[2]
    else:
        delimiter = ';'
    exprs = extract(sys.argv[1], delimiter)
    # exprs = [item for sublist in exprs for item in sublist]
    exprs = [i for i in exprs if i]
    frmt = []
    for i, expr in enumerate(exprs):
        frmt.append([i+1, expr[0], expr[1], expr[2]])
    with open('expressions_egg.csv', 'w') as f:
        # using csv.writer method from CSV package
        write = csv.writer(f)
        write.writerow(["ID", 'Expression', 'HalideResult', 'HalideTime'])
        write.writerows(frmt)
