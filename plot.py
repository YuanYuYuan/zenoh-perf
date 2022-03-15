#!/usr/bin/env python3

import pandas as pd
import plotly.express as px
import os
from glob import glob
from pathlib import Path
from tap import Tap
import plotly.graph_objects as go


class MyArgParser(Tap):
    # log dir
    log_dir: Path = Path('./outputs')
    # output dir
    out_dir: Path = Path('./plots')
    # experiment name
    exp_name: str = "t_pub_thr + t_sub_thr"


def load_data(dir: str, exp_tag: str = None) -> pd.DataFrame:
    data = pd.concat([
        pd.read_csv(f)
        for f in glob(os.path.join(dir, '*.log'))
    ])
    data = data.groupby(['size'], as_index=False).agg({'messages': ['mean', 'std']})
    data.columns = ['size', 'mean', 'std']
    if exp_tag:
        data['exp'] = exp_tag
    return data



def load_usage(dir: str, exp_tag: str = None) -> pd.DataFrame:
    data = pd.concat([
        load_single_usage(Path(file), mode)
        for mode in ['sub', 'pub']
        for file in glob(os.path.join(dir, mode, '*.txt'))
    ])
    if exp_tag:
        data['exp'] = exp_tag
    return data


def load_single_usage(file: Path, mode: str) -> pd.DataFrame:
    payload_size = int(file.stem)
    data = pd.read_csv(
        file,
        sep='\\s+',
        skiprows=1,
        names=['t', 'CPU', 'MEM', 'VMEM']
    )
    data['payload'] = payload_size
    data['MA_CPU'] = data['CPU'].rolling(30).mean()
    data['mode'] = mode
    return data


args = MyArgParser().parse_args()
os.makedirs(args.out_dir, exist_ok=True)
data = {
    ver: {
        'log': load_data(os.path.join(args.log_dir, f'{ver}/logs'), ver_name),
        'usage': load_usage(os.path.join(args.log_dir, f'{ver}/usages'), ver_name)
    }
    for (ver, ver_name) in [
        ['v5', '0.5.0-beta.9'],
        ['v6', '0.6.0-dev'],
    ]
}


def plot_usage(usage, output_dir):
    os.makedirs(output_dir, exist_ok=True)

    for payload_size in usage['payload'].unique():
        traces = []
        for (mode, cpu_color, mem_color) in [
            ['pub', '#33ccff', '#ff3399'],
            ['sub', '#3333ff', '#ff3333']
        ]:
            idx = (usage['payload'] == payload_size) & (usage['mode'] == mode)
            traces.append(go.Scatter(
                x=usage[idx]['t'],
                y=usage[idx]['MA_CPU'],
                marker=dict(color=cpu_color),
                name=mode + ': CPU',
                yaxis='y1',
            ))
            traces.append(go.Scatter(
                x=usage[idx]['t'],
                y=usage[idx]['MEM'],
                marker=dict(color=mem_color),
                name=mode + ': Memory',
                yaxis='y2',
            ))

        layout = go.Layout(
            title='CPU & Memory Usage, Payload size (bytes): %d' % payload_size,
            xaxis={
                'title': 'Time (sec)',
                'range': [0, 16],
                'dtick': 1,
            },
            yaxis={
                'title': 'CPU (%)',
                'range': [0, 100 * 2],
                'dtick': 20,
            },
            yaxis2={
                'title': 'Memory (MB)',
                'overlaying': 'y',
                'side': 'right',
                'range': [0, 16],
                'dtick': 1,
            },
            legend={
                'y': 1.3,
                'x': 0.92
            }
        )
        fig = go.Figure(
            data=traces,
            layout=layout
        )

        fig.write_image(os.path.join(
            output_dir,
            'payload-size-%07d.jpg' % payload_size
        ))
        #  fig.show()


for ver in ['v5', 'v6']:
    plot_usage(
        data[ver]['usage'],
        output_dir=os.path.join(args.out_dir, 'usages/' + ver)
    )

data['v5+v6'] = {
    'log': pd.concat([data[ver]['log'] for ver in ['v5', 'v6']])
}



#  v6['mean'] = v6['mean'] / v5['mean'] * 100

#  fig = px.line(
#      v6,
#      x='size',
#      y='mean',
#      log_x=True,
#      labels={
#          'size': 'Payload size (bytes)',
#          'mean': 'Msg/sec ratio',
#      },
#      title='Throughput Comparison (' + args.exp_name + ') Ratio (v6 / v5)'
#  )
#  file_name = os.path.join(args.out_dir, 'throughput-comparison-in-ratio')
#  fig.write_html(file_name + '.html')
#  fig.write_image(file_name + '.jpeg')
#  fig.show()


fig = px.line(
    data['v5+v6']['log'],
    x='size',
    y='mean',
    error_y='std',
    color='exp',
    log_x=True,
    log_y=True,
    labels={
        'size': 'Payload size (bytes)',
        'mean': 'Msg/sec',
        'exp': 'Zenoh Ver.'
    },
    title='Throughput Comparison (' + args.exp_name + ')'
)
file_name = os.path.join(args.out_dir, 'throughput-comparison')
fig.write_html(file_name + '.html')
fig.write_image(file_name + '.jpeg')
#  fig.show()
