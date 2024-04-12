let got_interrupt_data = false;
let interrupt_hide_zero_na_graphs = false;

function getLine(elem, key, run_data) {
    var data = JSON.parse(run_data);
    var cpus = data.data[0].per_cpu.length;
    var interrupt_type_datas = [];
    for (let cpu = 0; cpu < cpus; cpu++) {
        var x_time = [];
        var y_data = [];
        data.data.forEach(function (value, index, arr) {
            value.per_cpu.forEach(function (v, i, a) {
                if (v.cpu == cpu) {
                    x_time.push(value.time.TimeDiff);
                    y_data.push(v.count);
                }
            })
        })
        var interrupt_cpu_data = {
            name: `CPU ${cpu}`,
            x: x_time,
            y: y_data,
            type: 'scatter',
        };
        interrupt_type_datas.push(interrupt_cpu_data);
    }
    var title;
    title = `${key} (${data.id})`;
    var TESTER = elem;
    let limits = key_limits.get(key);
    var layout = {
        title: title,
        xaxis: {
            title: 'Time (s)',
        },
        yaxis: {
            title: 'Count',
            range: [limits.low, limits.high],
        }
    };
    Plotly.newPlot(TESTER, interrupt_type_datas, layout, { frameMargins: 0 });
}

function getLines(run, container_id, keys, run_data) {
    for (let i = 0; i < all_run_keys.length; i++) {
        let value = all_run_keys[i];
        var elem = document.createElement('div');
        elem.id = `interrupt-${run}-${value}`;
        elem.style.float = "none";
        addElemToNode(container_id, elem);
        emptyOrCallback(keys, interrupt_hide_zero_na_graphs, getLine, elem, value, run_data);
    }
}

let rewritten = false;

function interrupts(hide: boolean) {
    if (!rewritten) {
        for (const run of interrupts_raw_data.runs) {
            for (let key_idx = 0; key_idx < run.keys.length; key_idx++) {
                const orig_key = run.keys[key_idx];
                const orig_key_values = JSON.parse(run.key_values[orig_key]);

                const data_point = orig_key_values[0];
                let new_key = orig_key;
                if (data_point.interrupt_device != "") {
                    new_key = data_point.interrupt_device;
                } else if (data_point.interrupt_type != "") {
                    new_key = data_point.interrupt_type;
                }

                let max_count = 0;
                for (const d of orig_key_values) {
                    for (const c of d.per_cpu) {
                        max_count = Math.max(max_count, c.count);
                    }
                }

                const new_key_values = {
                    id: orig_key,
                    metadata: {
                        limits: {
                            low: 0,
                            high: max_count,
                            init_done: true,
                        },
                    },
                    data: orig_key_values,
                };

                run.keys[key_idx] = new_key;
                delete run.key_values[orig_key];
                run.key_values[new_key] = JSON.stringify(new_key_values);
            }
            run.keys.sort()
        }
        rewritten = true;
    }

    if (got_interrupt_data && hide == interrupt_hide_zero_na_graphs) {
        return;
    }
    interrupt_hide_zero_na_graphs = hide;
    clear_and_create('interrupts');
    form_graph_limits(interrupts_raw_data);
    for (let i = 0; i < interrupts_raw_data['runs'].length; i++) {
        let run_name = interrupts_raw_data['runs'][i]['name'];
        let elem_id = `${run_name}-interrupts-per-data`;
        let this_run_data = interrupts_raw_data['runs'][i];
        getLines(run_name, elem_id, this_run_data['keys'], this_run_data['key_values']);
    }
    got_interrupt_data = true;
}
